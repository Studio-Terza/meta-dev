use std::collections::{BTreeMap, BTreeSet};

use algebra::Monoid;

use crate::holes::Holes;
use crate::renaming::Renaming;

#[derive(Clone, Debug)]
pub struct Cluster<Node> {
    renamings: Vec<Renaming<Node>>,
}

#[derive(Clone, Debug)]
pub(crate) enum Placed<Node> {
    Leads(Node),
    Joins { leader: Node, renaming: Renaming<Node> },
}

impl<Node: Ord + Copy> Cluster<Node> {
    fn led(first: Node, joined: Vec<Renaming<Node>>) -> Cluster<Node> {
        Cluster { renamings: [Renaming::started(first, first)].into_iter().chain(joined).collect() }
    }

    pub fn members(&self) -> impl Iterator<Item = Node> + '_ {
        self.renamings.iter().map(Renaming::source)
    }

    #[must_use]
    pub fn holes(&self) -> Holes {
        let sources = self.renamings.iter().fold(BTreeMap::new(), |sources, renaming| renaming.tallied(sources));
        Holes::new(sources.values().filter(|found: &&BTreeSet<Node>| found.len() > 1).count())
    }
}

impl<Node: Ord + Copy> Placed<Node> {
    pub(crate) fn leader(&self) -> Option<Node> {
        match self {
            Placed::Leads(node) => Some(*node),
            Placed::Joins { .. } => None,
        }
    }

    pub(crate) fn gathered(placed: Vec<Placed<Node>>) -> Vec<Cluster<Node>> {
        let led: BTreeMap<Node, Vec<Renaming<Node>>> = Monoid::concat(placed.into_iter().map(Placed::contributed));
        led.into_iter().map(|(first, joined)| Cluster::led(first, joined)).collect()
    }

    fn contributed(self) -> BTreeMap<Node, Vec<Renaming<Node>>> {
        match self {
            Placed::Leads(node) => BTreeMap::from([(node, Vec::new())]),
            Placed::Joins { leader, renaming } => BTreeMap::from([(leader, vec![renaming])]),
        }
    }
}
