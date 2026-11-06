use crate::{Entry, NodeId, Title};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    identity: NodeId,
    entry: Entry,
    title: Option<Title>,
}

impl Node {
    #[must_use]
    pub fn new(identity: NodeId, entry: Entry) -> Node {
        Node { identity, entry, title: None }
    }

    #[must_use]
    pub fn titled(self, title: Title) -> Node {
        Node { title: Some(title), ..self }
    }

    #[must_use]
    pub fn identity(&self) -> NodeId {
        self.identity
    }

    #[must_use]
    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    #[must_use]
    pub fn title(&self) -> Option<&Title> {
        self.title.as_ref()
    }
}
