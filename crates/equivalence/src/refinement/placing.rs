use crate::cluster::Cluster;
use crate::renaming::Renaming;

#[derive(Debug)]
pub(super) struct Placing<Node> {
    clusters: Vec<Cluster<Node>>,
    pending: Option<Node>,
}

impl<Node: Ord + Copy> Placing<Node> {
    pub(super) fn starting_with(node: Node) -> Placing<Node> {
        Placing { clusters: Vec::new(), pending: Some(node) }
    }

    pub(super) fn offered(
        self,
        cluster: Cluster<Node>,
        rename: impl Fn(Node, Node) -> Option<Renaming<Node>>,
    ) -> Placing<Node> {
        let renaming = self.pending.and_then(|node| rename(cluster.first(), node));
        let pending = self.pending.filter(|_| renaming.is_none());
        let cluster = renaming.into_iter().fold(cluster, Cluster::joined);
        Placing { clusters: self.clusters.into_iter().chain([cluster]).collect(), pending }
    }

    pub(super) fn finished(self) -> Vec<Cluster<Node>> {
        self.clusters.into_iter().chain(self.pending.map(Cluster::started)).collect()
    }
}
