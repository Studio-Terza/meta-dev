use crate::{Entry, NodeId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    identity: NodeId,
    entry: Entry,
}

impl Node {
    #[must_use]
    pub fn new(identity: NodeId, entry: Entry) -> Node {
        Node { identity, entry }
    }

    #[must_use]
    pub fn identity(&self) -> NodeId {
        self.identity
    }

    #[must_use]
    pub fn entry(&self) -> &Entry {
        &self.entry
    }
}
