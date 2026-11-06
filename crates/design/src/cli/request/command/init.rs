use std::fmt;

use design_model::{Entered, Entry, Facet, NodeId, NodeRelation, Project, Provenance};
use design_store::{Linked, NewNode, ProjectRoot};
use file_ports::Files;
use store_ports::{Store, Written};

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

#[derive(Debug)]
struct Rooted {
    project: Project,
    root: NodeId,
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
        let consumed = self.consumes.map(|project| Rooted::found(store, project)).transpose()?;
        ground.configured(self.project.clone())?;
        let found = store.read(ProjectRoot::from(self.project.clone()))?;
        let root = found.map_or_else(|| Self::sown(store, &self.project), Ok)?;
        consumed
            .iter()
            .map(|consumed| {
                store.write(Linked::new(root, NodeRelation::Consumes, consumed.root, Provenance::from(Entered::Init)))
            })
            .collect::<Result<Vec<Written>, _>>()?;
        lock.end()?;
        Ok(Initialised { root: Rooted { project: self.project, root }, consumed })
    }

    fn sown(store: &impl Store, project: &Project) -> Result<NodeId, Failure> {
        let entry = Entry::new(project.clone(), Facet::Project, Provenance::from(Entered::Init));
        store.write(NewNode::from(entry))?;
        store.read(ProjectRoot::from(project.clone()))?.ok_or_else(|| Failure::Unrooted(project.clone()))
    }
}

impl Rooted {
    fn found(store: &impl Store, project: Project) -> Result<Rooted, Failure> {
        let root = store.read(ProjectRoot::from(project.clone()))?;
        root.map(|root| Rooted { project: project.clone(), root }).ok_or(Failure::Unrooted(project))
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
