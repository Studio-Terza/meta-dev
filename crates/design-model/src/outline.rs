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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sketch {
    node: NodeId,
    project: Project,
    facet: Facet,
    stage: Stage,
}

impl Sketch {
    #[must_use]
    pub fn new(node: NodeId, project: Project, facet: Facet, stage: Stage) -> Sketch {
        Sketch { node, project, facet, stage }
    }

    #[must_use]
    pub fn described(self, basis: Basis, title: Option<Title>, parent: Option<NodeId>) -> Outline {
        let Sketch { node, project, facet, stage } = self;
        Outline { node, project, facet, stage, basis, title, parent }
    }
}

impl Outline {
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
