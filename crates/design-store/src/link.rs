use design_model::{NodeId, NodeRelation, Provenance};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Link {
    relation: NodeRelation,
    target: NodeId,
    provenance: Provenance,
}

impl Link {
    pub(crate) fn new(relation: NodeRelation, target: NodeId, provenance: Provenance) -> Link {
        Link { relation, target, provenance }
    }

    #[must_use]
    pub fn relation(self) -> NodeRelation {
        self.relation
    }

    #[must_use]
    pub fn target(self) -> NodeId {
        self.target
    }

    #[must_use]
    pub fn provenance(self) -> Provenance {
        self.provenance
    }
}
