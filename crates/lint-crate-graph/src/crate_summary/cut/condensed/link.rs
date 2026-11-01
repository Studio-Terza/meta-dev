use super::spectrum::Weight;
use super::tangle_index::TangleIndex;

#[derive(Clone, Copy, Debug)]
pub struct Link {
    target: TangleIndex,
    weight: Weight,
}

impl Link {
    pub fn new(target: TangleIndex, weight: Weight) -> Link {
        Link { target, weight }
    }

    pub fn target(self) -> TangleIndex {
        self.target
    }

    pub fn weight(self) -> Weight {
        self.weight
    }
}
