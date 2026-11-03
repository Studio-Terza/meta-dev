#[derive(Debug)]
pub struct Frame<Index, Pending> {
    node: Index,
    pending: Pending,
}

impl<Index: Copy, Pending: Iterator<Item = Index>> Frame<Index, Pending> {
    pub fn entering(node: Index, pending: Pending) -> Frame<Index, Pending> {
        Frame { node, pending }
    }

    pub fn node(&self) -> Index {
        self.node
    }

    pub fn next_neighbour(&mut self) -> Option<Index> {
        self.pending.next()
    }
}
