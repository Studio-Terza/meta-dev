use crate::{NodeId, NodeRelation, Role};

const ENDS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeEdge {
    source: NodeId,
    relation: NodeRelation,
    target: NodeId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the edge would have a core node depend on a secondary one")]
pub struct Inversion {
    edge: NodeEdge,
}

impl NodeEdge {
    #[must_use]
    pub fn new(source: NodeId, relation: NodeRelation, target: NodeId) -> NodeEdge {
        NodeEdge { source, relation, target }
    }

    #[must_use]
    pub fn ends(self) -> [NodeId; ENDS] {
        [self.source, self.target]
    }

    #[must_use]
    pub fn relation(self) -> NodeRelation {
        self.relation
    }

    pub fn upheld(self, source: Role, target: Role) -> Result<NodeEdge, Inversion> {
        self.leaning(source, target)
            .filter(|roles| *roles == [Role::Core, Role::Secondary])
            .map_or(Ok(self), |_inverted| Err(Inversion { edge: self }))
    }

    fn leaning(self, source: Role, target: Role) -> Option<[Role; ENDS]> {
        match self.relation {
            NodeRelation::Refines | NodeRelation::Consumes => Some([source, target]),
            NodeRelation::Supersedes => None,
        }
    }
}

impl Inversion {
    #[must_use]
    pub fn edge(self) -> NodeEdge {
        self.edge
    }
}
