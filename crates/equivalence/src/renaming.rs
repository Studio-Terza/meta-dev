use std::collections::{BTreeMap, BTreeSet};

use algebra::Act;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Renaming<Node> {
    from: Node,
    pairs: BTreeMap<Node, Node>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pair<Node> {
    from: Option<Node>,
    to: Option<Node>,
}

#[derive(Clone, Copy, Debug)]
struct Renamed<Node> {
    from: Node,
    to: Node,
}

impl<Node: Ord + Copy> Renaming<Node> {
    #[must_use]
    pub fn started(from: Node, to: Node) -> Renaming<Node> {
        Renaming { from, pairs: BTreeMap::from([(from, to)]) }
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
}

impl<Node: Ord + Copy> Act<Vec<Pair<Node>>> for Renaming<Node> {
    fn acted(self, by: Vec<Pair<Node>>) -> Option<Renaming<Node>> {
        by.into_iter().try_fold(self, |mut renaming, pair| {
            let claimed = pair.claimed(&renaming.pairs)?;
            renaming.pairs.extend(claimed);
            Some(renaming)
        })
    }
}

impl<Node: Ord + Copy> Pair<Node> {
    #[must_use]
    pub fn new(from: Option<Node>, to: Option<Node>) -> Pair<Node> {
        Pair { from, to }
    }

    fn claimed(self, pairs: &BTreeMap<Node, Node>) -> Option<BTreeMap<Node, Node>> {
        let unpaired = (self.from.is_none() == self.to.is_none()).then(BTreeMap::new);
        let renamed = self.from.zip(self.to).map(|(from, to)| Renamed { from, to });
        renamed.map_or(unpaired, |both| both.fresh(pairs))
    }
}

impl<Node: Ord + Copy> Renamed<Node> {
    fn fresh(self, pairs: &BTreeMap<Node, Node>) -> Option<BTreeMap<Node, Node>> {
        let unclaimed = pairs.values().all(|taken| *taken != self.to).then(|| BTreeMap::from([(self.from, self.to)]));
        pairs.get(&self.from).map_or(unclaimed, |known| (*known == self.to).then(BTreeMap::new))
    }
}
