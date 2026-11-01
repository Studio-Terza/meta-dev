#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeIndex(usize);

impl From<usize> for NodeIndex {
    fn from(index: usize) -> NodeIndex {
        NodeIndex(index)
    }
}

impl From<NodeIndex> for usize {
    fn from(node: NodeIndex) -> usize {
        node.0
    }
}

impl NodeIndex {
    #[must_use]
    pub fn index(self) -> usize {
        self.0
    }
}
