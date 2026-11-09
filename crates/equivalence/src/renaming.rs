use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Renaming<Node> {
    from: Node,
    pairs: BTreeMap<Node, Node>,
}

impl<Node: Ord + Copy> Renaming<Node> {
    pub(crate) fn started(from: Node, to: Node) -> Renaming<Node> {
        Renaming { from, pairs: BTreeMap::from([(from, to)]) }
    }

    pub(crate) fn through(self, from: Option<Node>, to: Option<Node>) -> Option<Renaming<Node>> {
        match from.zip(to) {
            Some((source, target)) => self.paired(source, target),
            None => (from.is_none() == to.is_none()).then_some(self),
        }
    }

    pub(crate) fn paired(self, from: Node, to: Node) -> Option<Renaming<Node>> {
        match self.pairs.get(&from).copied() {
            Some(known) => (known == to).then_some(self),
            None => self.unclaimed(to).map(|renaming| renaming.claimed(from, to)),
        }
    }

    pub(crate) fn tallied(&self, sources: BTreeMap<Node, BTreeSet<Node>>) -> BTreeMap<Node, BTreeSet<Node>> {
        self.pairs.iter().fold(sources, |mut tally, (from, to)| {
            tally.entry(*to).or_insert_with(|| BTreeSet::from([*to])).insert(*from);
            tally
        })
    }

    pub(crate) fn source(&self) -> Node {
        self.from
    }

    fn unclaimed(self, to: Node) -> Option<Renaming<Node>> {
        Some(self).filter(|renaming| renaming.pairs.values().all(|taken| *taken != to))
    }

    fn claimed(mut self, from: Node, to: Node) -> Renaming<Node> {
        self.pairs.insert(from, to);
        self
    }
}
