use design_model::{AnyNode, Kin};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Relative {
    kin: Kin,
    node: AnyNode,
}

impl Relative {
    pub(crate) fn new(kin: Kin, node: AnyNode) -> Relative {
        Relative { kin, node }
    }

    #[must_use]
    pub fn kin(&self) -> Kin {
        self.kin
    }

    #[must_use]
    pub fn node(&self) -> &AnyNode {
        &self.node
    }
}
