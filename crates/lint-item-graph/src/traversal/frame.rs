#[derive(Debug)]
pub struct Frame<Index, Pending> {
    node: Index,
    pending: Pending,
}

impl<Index: Copy, Pending: Iterator<Item = Index>> Frame<Index, Pending> {
    pub const fn entering(node: Index, pending: Pending) -> Self {
        Self { node, pending }
    }

    pub const fn node(&self) -> Index {
        self.node
    }

    pub fn next_neighbour(&mut self) -> Option<Index> {
        self.pending.next()
    }
}
