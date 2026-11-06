use design_model::{NodeId, NodeRelation};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Link {
    relation: NodeRelation,
    target: NodeId,
}

impl Link {
    pub(crate) fn new(relation: NodeRelation, target: NodeId) -> Link {
        Link { relation, target }
    }

    #[must_use]
    pub fn relation(self) -> NodeRelation {
        self.relation
    }

    #[must_use]
    pub fn target(self) -> NodeId {
        self.target
    }
}
