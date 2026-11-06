use design_model::{Kin, Node};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Relative {
    kin: Kin,
    node: Node,
}

impl Relative {
    pub(crate) fn new(kin: Kin, node: Node) -> Relative {
        Relative { kin, node }
    }

    #[must_use]
    pub fn kin(&self) -> Kin {
        self.kin
    }

    #[must_use]
    pub fn node(&self) -> &Node {
        &self.node
    }
}
