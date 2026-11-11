use design_model::{Inversion, Misstaged, NodeId, Segment};
use store_ports::StoreFailure;

use design_closure::Cycle;
use design_closure::Refuses;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the directives leave segments of their job unstructured")]
pub struct Unpaid {
    first: Segment,
    rest: Vec<Segment>,
}

#[derive(Debug, thiserror::Error)]
pub enum Refusal {
    #[error("the edge would close a cycle of its relation")]
    Cyclic(Cycle),
    #[error("the superseding node refines the node it would supersede")]
    Ancestral { old: NodeId, by: NodeId },
    #[error("{0}")]
    Inverted(#[from] Inversion),
    #[error("{0}")]
    Misstaged(#[from] Misstaged),
    #[error("{0}")]
    Unpaid(#[from] Unpaid),
    #[error("{0}")]
    Store(#[from] StoreFailure),
}

impl Refuses for Refusal {
    fn cyclic(cycle: Cycle) -> Refusal {
        Refusal::Cyclic(cycle)
    }
}

impl Unpaid {
    pub(crate) fn new(first: Segment, rest: Vec<Segment>) -> Unpaid {
        Unpaid { first, rest }
    }

    #[must_use]
    pub fn first(&self) -> &Segment {
        &self.first
    }

    #[must_use]
    pub fn rest(&self) -> &[Segment] {
        &self.rest
    }
}
