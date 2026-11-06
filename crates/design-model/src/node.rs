use crate::lifecycle::{Live, Refinable};
use crate::{
    AnyStage, Artifact, Condition, Dropped, Entry, Misstaged, NodeId, Reason, Refining, Role, Terminal, Title, Vague,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node<S> {
    identity: NodeId,
    entry: Entry,
    title: Option<Title>,
    role: Role,
    stage: S,
}

pub type AnyNode = Node<AnyStage>;

impl Node<Vague> {
    #[must_use]
    pub fn new(identity: NodeId, entry: Entry) -> Node<Vague> {
        Node { identity, entry, title: None, role: Role::Core, stage: Vague }
    }
}

impl<S: Refinable> Node<S> {
    #[must_use]
    pub fn refine(self) -> Node<Refining> {
        self.staged(Refining)
    }
}

impl Node<Refining> {
    #[must_use]
    pub fn specify(self, artifact: Artifact, condition: Condition) -> Node<Terminal> {
        self.staged(Terminal::new(artifact, condition))
    }
}

impl Node<Live> {
    #[must_use]
    pub fn drop(self, reason: Reason) -> Node<Dropped> {
        self.staged(Dropped::from(reason))
    }
}

impl AnyNode {
    #[must_use]
    pub fn loaded(identity: NodeId, entry: Entry, role: Role, stage: AnyStage) -> AnyNode {
        Node { identity, entry, title: None, role, stage }
    }

    pub fn narrowed<S: TryFrom<AnyStage, Error = AnyStage>>(self) -> Result<Node<S>, Misstaged> {
        let Node { identity, entry, title, role, stage } = self;
        S::try_from(stage)
            .map(|stage| Node { identity, entry, title, role, stage })
            .map_err(|stage| Misstaged::new(identity, stage.stage()))
    }
}

impl<S: Into<AnyStage>> Node<S> {
    #[must_use]
    pub fn any(self) -> AnyNode {
        let Node { identity, entry, title, role, stage } = self;
        Node { identity, entry, title, role, stage: stage.into() }
    }
}

impl<S> Node<S> {
    #[must_use]
    pub fn titled(self, title: Title) -> Node<S> {
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

    #[must_use]
    pub fn role(&self) -> Role {
        self.role
    }

    #[must_use]
    pub fn stage(&self) -> &S {
        &self.stage
    }

    fn staged<T>(self, stage: T) -> Node<T> {
        let Node { identity, entry, title, role, stage: _ } = self;
        Node { identity, entry, title, role, stage }
    }
}
