use crate::{Basis, Facet, NodeId, Project, Stage, Title};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outline {
    node: NodeId,
    project: Project,
    facet: Facet,
    stage: Stage,
    basis: Basis,
    title: Option<Title>,
    parent: Option<NodeId>,
}

impl Outline {
    #[must_use]
    pub fn new(node: NodeId, project: Project, facet: Facet, stage: Stage) -> Outline {
        Outline { node, project, facet, stage, basis: Basis::Vague, title: None, parent: None }
    }

    #[must_use]
    pub fn based(self, basis: Basis) -> Outline {
        Outline { basis, ..self }
    }

    #[must_use]
    pub fn titled(self, title: Option<Title>) -> Outline {
        Outline { title, ..self }
    }

    #[must_use]
    pub fn parented(self, parent: Option<NodeId>) -> Outline {
        Outline { parent, ..self }
    }

    #[must_use]
    pub fn node(&self) -> NodeId {
        self.node
    }

    #[must_use]
    pub fn project(&self) -> &Project {
        &self.project
    }

    #[must_use]
    pub fn facet(&self) -> Facet {
        self.facet
    }

    #[must_use]
    pub fn stage(&self) -> Stage {
        self.stage
    }

    #[must_use]
    pub fn basis(&self) -> Basis {
        self.basis
    }

    #[must_use]
    pub fn title(&self) -> Option<&Title> {
        self.title.as_ref()
    }

    #[must_use]
    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }
}
