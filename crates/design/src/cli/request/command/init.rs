use std::fmt;

use design_model::{Entered, Entry, Facet, Node, NodeId, Project, Provenance, Vague};
use design_store::{NextNode, Planting, ProjectRoot, Resolved};
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
    Found(Project, Resolved),
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
        let root = Root::resolved(store, self.project)?;
        let rooted = root.rooted();
        let planting = consumed.iter().fold(root.planting(), |planting, (_, resolved)| planting.consuming(*resolved));
        Ok(Plan { planting, initialised: Initialised { root: rooted, consumed: consumed.map(|(rooted, _)| rooted) } })
    }
}

impl Root {
    fn resolved(store: &impl Store, project: Project) -> Result<Root, Failure> {
        match store.read(ProjectRoot::from(project.clone()))? {
            Some(root) => Ok(Root::Found(project, root)),
            None => Ok(Root::Sown(Node::new(
                store.read(NextNode)?,
                Entry::new(project, Facet::Project, Provenance::from(Entered::Init)),
            ))),
        }
    }

    fn rooted(&self) -> Rooted {
        match self {
            Root::Found(project, root) => Rooted { project: project.clone(), root: root.identity() },
            Root::Sown(node) => Rooted { project: node.entry().project().clone(), root: node.identity() },
        }
    }

    fn planting(self) -> Planting {
        match self {
            Root::Found(_, root) => Planting::found(root),
            Root::Sown(node) => Planting::sown(node),
        }
    }
}

impl Rooted {
    fn found(store: &impl Store, project: Project) -> Result<(Rooted, Resolved), Failure> {
        let root = store.read(ProjectRoot::from(project.clone()))?.ok_or_else(|| Failure::Unrooted(project.clone()))?;
        Ok((Rooted { project, root: root.identity() }, root))
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
