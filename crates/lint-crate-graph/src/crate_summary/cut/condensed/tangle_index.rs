#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TangleIndex(usize);

impl From<usize> for TangleIndex {
    fn from(index: usize) -> TangleIndex {
        TangleIndex(index)
    }
}

impl From<TangleIndex> for usize {
    fn from(tangle: TangleIndex) -> usize {
        tangle.0
    }
}

impl TangleIndex {
    pub fn index(self) -> usize {
        self.0
    }
}
