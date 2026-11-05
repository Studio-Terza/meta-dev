use crate::placed::Placed;
use crate::renaming::Renaming;

#[derive(Clone, Debug)]
pub(crate) struct Cluster<Node> {
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

    pub(crate) fn placed(self) -> impl Iterator<Item = Placed<Node>> {
        let first = self.first;
        self.renamings.into_iter().map(move |renaming| Placed::new(first, renaming.source(), renaming))
    }
}
