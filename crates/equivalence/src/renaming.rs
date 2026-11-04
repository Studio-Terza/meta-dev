use std::collections::BTreeMap;

use crate::holes::Holes;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Renaming<Node> {
    pairs: BTreeMap<Node, Node>,
}

impl<Node: Ord + Copy> Renaming<Node> {
    pub(crate) fn started(from: Node, to: Node) -> Renaming<Node> {
        Renaming { pairs: BTreeMap::from([(from, to)]) }
    }

    pub(crate) fn through(self, from: Option<Node>, to: Option<Node>) -> Option<Renaming<Node>> {
        match from.zip(to) {
            Some((from, to)) => self.paired(from, to),
            None => (from.is_none() == to.is_none()).then_some(self),
        }
    }

    pub(crate) fn paired(self, from: Node, to: Node) -> Option<Renaming<Node>> {
        match self.pairs.get(&from).copied() {
            Some(known) => (known == to).then_some(self),
            None => self.unclaimed(to).map(|renaming| renaming.claimed(from, to)),
        }
    }

    #[must_use]
    pub fn holes(&self) -> Holes {
        Holes::new(self.pairs.iter().filter(|(from, to)| from != to).count())
    }

    fn unclaimed(self, to: Node) -> Option<Renaming<Node>> {
        Some(self).filter(|renaming| renaming.pairs.values().all(|taken| *taken != to))
    }

    fn claimed(mut self, from: Node, to: Node) -> Renaming<Node> {
        self.pairs.insert(from, to);
        self
    }
}
