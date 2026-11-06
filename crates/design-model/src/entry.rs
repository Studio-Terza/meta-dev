use crate::{Facet, Project, Provenance, Stage};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    project: Project,
    facet: Facet,
    stage: Stage,
    provenance: Provenance,
}

impl Entry {
    #[must_use]
    pub fn new(project: Project, facet: Facet, stage: Stage, provenance: Provenance) -> Entry {
        Entry { project, facet, stage, provenance }
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
    pub fn provenance(&self) -> Provenance {
        self.provenance
    }
}
