use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::group::Group;
use crate::holes::Holes;
use crate::member::Keyed;
use crate::renaming::Renaming;

#[derive(Clone, Debug)]
pub struct Placed<Node> {
    first: Node,
    node: Node,
    renaming: Renaming<Node>,
}

impl<Node: Ord + Copy> Placed<Node> {
    pub(crate) fn new(first: Node, node: Node, renaming: Renaming<Node>) -> Placed<Node> {
        Placed { first, node, renaming }
    }

    #[must_use]
    pub fn node(&self) -> Node {
        self.node
    }
}

impl<Node: Ord + Copy + fmt::Debug> Keyed for Placed<Node> {
    type Key = Node;

    fn key(&self) -> Node {
        self.first
    }
}

impl<Node: Ord + Copy + fmt::Debug> Group<Placed<Node>> {
    #[must_use]
    pub fn holes(&self) -> Holes {
        let sources = self.members().iter().fold(BTreeMap::new(), |sources, placed| placed.renaming.tallied(sources));
        Holes::new(sources.values().filter(|found: &&BTreeSet<Node>| found.len() > 1).count())
    }
}
