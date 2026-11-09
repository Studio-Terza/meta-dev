use std::collections::{BTreeMap, BTreeSet};

use crate::holes::Holes;
use crate::renaming::Renaming;

#[derive(Clone, Debug)]
pub struct Cluster<Node> {
    first: Node,
    renamings: Vec<Renaming<Node>>,
}

impl<Node: Ord + Copy> Cluster<Node> {
    pub(crate) fn started(first: Node) -> Cluster<Node> {
        Cluster { first, renamings: vec![Renaming::started(first, first)] }
    }

    pub(crate) fn joined(mut self, renaming: Renaming<Node>) -> Cluster<Node> {
        self.renamings.push(renaming);
        self
    }

    pub(crate) fn first(&self) -> Node {
        self.first
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

#[derive(Debug)]
pub(crate) struct Placing<Node> {
    clusters: Vec<Cluster<Node>>,
    pending: Option<Node>,
}

impl<Node: Ord + Copy> Placing<Node> {
    pub(crate) fn starting_with(node: Node) -> Placing<Node> {
        Placing { clusters: Vec::new(), pending: Some(node) }
    }

    pub(crate) fn offered(
        self,
        cluster: Cluster<Node>,
        rename: impl Fn(Node, Node) -> Option<Renaming<Node>>,
    ) -> Placing<Node> {
        let renaming = self.pending.and_then(|node| rename(cluster.first(), node));
        let pending = self.pending.filter(|_| renaming.is_none());
        let joined = renaming.into_iter().fold(cluster, Cluster::joined);
        Placing { clusters: self.clusters.into_iter().chain([joined]).collect(), pending }
    }

    pub(crate) fn finished(self) -> Vec<Cluster<Node>> {
        self.clusters.into_iter().chain(self.pending.map(Cluster::started)).collect()
    }
}
