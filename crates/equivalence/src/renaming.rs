use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Renaming<Node> {
    from: Node,
    pairs: BTreeMap<Node, Node>,
}

impl<Node: Ord + Copy> Renaming<Node> {
    pub(crate) fn started(from: Node, to: Node) -> Self {
        Self { from, pairs: BTreeMap::from([(from, to)]) }
    }

    pub(crate) fn through(self, from: Option<Node>, to: Option<Node>) -> Option<Self> {
        match from.zip(to) {
            Some((from, to)) => self.paired(from, to),
            None => (from.is_none() == to.is_none()).then_some(self),
        }
    }

    pub(crate) fn paired(self, from: Node, to: Node) -> Option<Self> {
        match self.pairs.get(&from).copied() {
            Some(known) => (known == to).then_some(self),
            None => self.unclaimed(to).map(|renaming| renaming.claimed(from, to)),
        }
    }

    pub(crate) fn tallied(&self, sources: BTreeMap<Node, BTreeSet<Node>>) -> BTreeMap<Node, BTreeSet<Node>> {
        self.pairs.iter().fold(sources, |mut sources, (from, to)| {
            sources.entry(*to).or_insert_with(|| BTreeSet::from([*to])).insert(*from);
            sources
        })
    }

    pub(crate) const fn source(&self) -> Node {
        self.from
    }

    fn unclaimed(self, to: Node) -> Option<Self> {
        Some(self).filter(|renaming| renaming.pairs.values().all(|taken| *taken != to))
    }

    fn claimed(mut self, from: Node, to: Node) -> Self {
        self.pairs.insert(from, to);
        self
    }
}
