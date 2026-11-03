mod frame;

use std::collections::BTreeSet;
use std::{iter, option};

use frame::Frame;

#[derive(Debug)]
pub struct Traversal<Index, Roots, Neighbours, Pending> {
    neighbours: Neighbours,
    roots: Roots,
    seen: BTreeSet<Index>,
    path: Vec<Frame<Index, Pending>>,
}

impl<Index, Roots, Neighbours, Pending> Traversal<Index, Roots, Neighbours, Pending>
where
    Index: Copy + Ord,
    Roots: Iterator<Item = Index>,
    Neighbours: Fn(Index) -> Pending,
    Pending: Iterator<Item = Index>,
{
    #[must_use]
    pub fn trees(roots: impl IntoIterator<IntoIter = Roots>, neighbours: Neighbours) -> Vec<Vec<Index>> {
        Traversal { neighbours, roots: roots.into_iter(), seen: BTreeSet::new(), path: Vec::new() }.collect()
    }

    fn enter(&mut self, node: Index) {
        self.path.push(Frame::entering(node, (self.neighbours)(node)));
    }

    fn step(&mut self) -> Option<option::IntoIter<Index>> {
        let next = self.path.last_mut()?.next_neighbour();
        Some(self.advanced(next))
    }

    fn advanced(&mut self, next: Option<Index>) -> option::IntoIter<Index> {
        match next {
            Some(child) => self.descended(child),
            None => self.path.pop().map(|frame| frame.node()).into_iter(),
        }
    }

    fn descended(&mut self, child: Index) -> option::IntoIter<Index> {
        Some(child).filter(|child| self.seen.insert(*child)).into_iter().for_each(|child| self.enter(child));
        None.into_iter()
    }
}

impl<Index, Roots, Neighbours, Pending> Iterator for Traversal<Index, Roots, Neighbours, Pending>
where
    Index: Copy + Ord,
    Roots: Iterator<Item = Index>,
    Neighbours: Fn(Index) -> Pending,
    Pending: Iterator<Item = Index>,
{
    type Item = Vec<Index>;

    fn next(&mut self) -> Option<Vec<Index>> {
        let seen = &mut self.seen;
        let root = self.roots.find(|root| seen.insert(*root))?;
        self.enter(root);
        Some(iter::from_fn(|| self.step()).flatten().collect())
    }
}
