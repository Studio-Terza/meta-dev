#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TangleIndex(usize);

impl From<usize> for TangleIndex {
    fn from(index: usize) -> TangleIndex {
        TangleIndex(index)
    }
}

impl TangleIndex {
    pub fn into_inner(self) -> usize {
        self.0
    }
}
