use std::fmt;

use design_model::{Entered, Entry, Facet, Node, NodeId, NodeRelation, Project, Provenance, Vague};
use design_store::{Linked, NextNode, Planting, ProjectRoot, ShownNode};
use file_ports::Files;
use store_ports::Store;

use crate::cli::failure::Failure;
use crate::cli::rendering::Id;
use crate::cli::request::{Ground, Token, Valued};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Init {
    project: Project,
    consumes: Option<Project>,
}

#[derive(Debug)]
pub struct Initialised {
    root: Rooted,
    consumed: Option<Rooted>,
}

#[derive(Clone, Debug)]
struct Rooted {
    project: Project,
    root: NodeId,
}

#[derive(Debug)]
enum Root {
    Found(Rooted),
    Sown(Node<Vague>),
}

#[derive(Debug)]
struct Plan {
    initialised: Initialised,
    planting: Planting,
}

impl TryFrom<&[Token]> for Init {
    type Error = Failure;

    fn try_from(rest: &[Token]) -> Result<Init, Failure> {
        let (consuming, naming): (Vec<&Token>, Vec<&Token>) =
            rest.iter().partition(|token| token.valued(Valued::Consumes).is_some());
        let [project] = <[&Token; 1]>::try_from(naming).map_err(|_named| Failure::Usage)?;
        let consumed = <[&Token; 1]>::try_from(consuming)
            .map(|[consumed]| Some(consumed))
            .or_else(|consuming| consuming.first().map_or(Ok(None), |_several| Err(Failure::Usage)))?;
        Init::new(project.project()?, consumed.map(Token::project).transpose()?)
    }
}

impl Init {
    fn new(project: Project, consumes: Option<Project>) -> Result<Init, Failure> {
        let itself = consumes.iter().find(|consumed| **consumed == project).cloned();
        itself.map(Failure::SelfConsuming).map_or(Ok(Init { project, consumes }), Err)
    }

    pub fn planted<F: Files, S: Store>(self, ground: &Ground<'_, F, S>) -> Result<Initialised, Failure> {
        let store = ground.store();
        let lock = store.lock()?;
        let plan = self.plan(store)?;
        store.write(plan.planting)?;
        ground.configured(plan.initialised.root.project.clone())?;
        lock.end()?;
        Ok(plan.initialised)
    }

    fn plan(self, store: &impl Store) -> Result<Plan, Failure> {
        let consumed = self.consumes.map(|project| Rooted::found(store, project)).transpose()?;
        let sown = Root::resolved(store, self.project)?;
        let root = sown.rooted();
        let linked = consumed.iter().map(|consumed| root.consuming(consumed)).next();
        Ok(Plan { planting: Planting::new(sown.sown(), linked), initialised: Initialised { root, consumed } })
    }
}

impl Root {
    fn resolved(store: &impl Store, project: Project) -> Result<Root, Failure> {
        match store.read(ProjectRoot::from(project.clone()))? {
            Some(root) => Rooted::placed(store, project, root).map(Root::Found),
            None => Ok(Root::Sown(Node::new(
                store.read(NextNode)?,
                Entry::new(project, Facet::Project, Provenance::from(Entered::Init)),
            ))),
        }
    }

    fn rooted(&self) -> Rooted {
        match self {
            Root::Found(rooted) => rooted.clone(),
            Root::Sown(node) => Rooted { project: node.entry().project().clone(), root: node.identity() },
        }
    }

    fn sown(self) -> Option<Node<Vague>> {
        match self {
            Root::Found(_) => None,
            Root::Sown(node) => Some(node),
        }
    }
}

impl Rooted {
    fn found(store: &impl Store, project: Project) -> Result<Rooted, Failure> {
        let root = store.read(ProjectRoot::from(project.clone()))?;
        root.ok_or_else(|| Failure::Unrooted(project.clone())).and_then(|root| Rooted::placed(store, project, root))
    }

    fn placed(store: &impl Store, project: Project, root: NodeId) -> Result<Rooted, Failure> {
        store.read(ShownNode::from(root))?.ok_or(Failure::Unplaced(root))?;
        Ok(Rooted { project, root })
    }

    fn consuming(&self, consumed: &Rooted) -> Linked {
        Linked::new(self.root, NodeRelation::Consumes, consumed.root, Provenance::from(Entered::Init))
    }
}

impl fmt::Display for Initialised {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "root {} project {}", Id::from(self.root.root), self.root.project)?;
        self.consumed.iter().try_for_each(|consumed| {
            writeln!(formatter, "consumes {} project {}", Id::from(consumed.root), consumed.project)
        })
    }
}
