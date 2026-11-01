use std::vec::IntoIter;

#[derive(Debug)]
pub struct Frame<Index> {
    node: Index,
    pending: IntoIter<Index>,
}

impl<Index: Copy> Frame<Index> {
    pub fn entering(node: Index, neighbours: Vec<Index>) -> Frame<Index> {
        Frame { node, pending: neighbours.into_iter() }
    }

    pub fn node(&self) -> Index {
        self.node
    }

    pub fn next_neighbour(&mut self) -> Option<Index> {
        self.pending.next()
    }
}
