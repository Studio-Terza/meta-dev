#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeIndex(usize);

impl From<usize> for NodeIndex {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl NodeIndex {
    #[must_use]
    pub fn into_inner(self) -> usize {
        self.0
    }
}
