mod frame;

use std::collections::BTreeSet;
use std::iter;
use std::vec::IntoIter;

use frame::Frame;

#[derive(Debug)]
pub struct Traversal<Index, Neighbours> {
    neighbours: Neighbours,
    roots: IntoIter<Index>,
    seen: BTreeSet<Index>,
    path: Vec<Frame<Index>>,
}

impl<Index: Copy + Ord, Neighbours: Fn(Index) -> Vec<Index>> Traversal<Index, Neighbours> {
    #[must_use]
    pub fn trees(roots: Vec<Index>, neighbours: Neighbours) -> Vec<Vec<Index>> {
        Traversal { neighbours, roots: roots.into_iter(), seen: BTreeSet::new(), path: Vec::new() }.collect()
    }

    fn enter(&mut self, node: Index) {
        self.path.push(Frame::entering(node, (self.neighbours)(node)));
    }

    fn step(&mut self) -> Option<Vec<Index>> {
        let next = self.path.last_mut()?.next_neighbour();
        Some(match next {
            Some(child) => {
                Some(child).filter(|child| self.seen.insert(*child)).into_iter().for_each(|child| self.enter(child));
                Vec::new()
            }
            None => self.path.pop().map(|frame| frame.node()).into_iter().collect(),
        })
    }
}

impl<Index: Copy + Ord, Neighbours: Fn(Index) -> Vec<Index>> Iterator for Traversal<Index, Neighbours> {
    type Item = Vec<Index>;

    fn next(&mut self) -> Option<Vec<Index>> {
        let seen = &mut self.seen;
        let root = self.roots.find(|root| seen.insert(*root))?;
        self.enter(root);
        Some(iter::from_fn(|| self.step()).flatten().collect())
    }
}
