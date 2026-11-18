use crate::lifecycle::{Live, Refinable};
use crate::{
    AnyStage, Artifact, Basis, Casting, Condition, Dropped, Entry, Misstaged, NodeId, Reason, Refining, Role, Terminal,
    Title, Vague,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node<S> {
    identity: NodeId,
    entry: Entry,
    title: Option<Title>,
    casting: Casting,
    stage: S,
    basis: Basis,
}

pub type AnyNode = Node<AnyStage>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Loaded {
    identity: NodeId,
    entry: Entry,
    casting: Casting,
    stage: AnyStage,
}

impl Loaded {
    #[must_use]
    pub fn described(self, basis: Basis, title: Option<Title>) -> AnyNode {
        let Loaded { identity, entry, casting, stage } = self;
        Node { identity, entry, title, casting, stage, basis }
    }
}

impl Node<Vague> {
    #[must_use]
    pub fn new(identity: NodeId, entry: Entry) -> Node<Vague> {
        let casting = Casting::new(Role::Core, entry.provenance());
        Node { identity, entry, title: None, casting, stage: Vague, basis: Basis::Vague }
    }

    #[must_use]
    pub fn proposed(identity: NodeId, entry: Entry, parent: Casting, title: Title) -> Node<Vague> {
        let casting = parent.inherited(entry.provenance());
        Node { identity, entry, title: Some(title), casting, stage: Vague, basis: Basis::Vague }
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
    pub fn loaded(identity: NodeId, entry: Entry, casting: Casting, stage: AnyStage) -> Loaded {
        Loaded { identity, entry, casting, stage }
    }

    pub fn narrowed<S: TryFrom<AnyStage, Error = AnyStage>>(self) -> Result<Node<S>, Misstaged> {
        let Node { identity, entry, title, casting, stage, basis } = self;
        S::try_from(stage)
            .map(|narrowed| Node { identity, entry, title, casting, stage: narrowed, basis })
            .map_err(|refused| Misstaged::new(identity, refused.stage()))
    }
}

impl<S: Into<AnyStage>> Node<S> {
    #[must_use]
    pub fn any(self) -> AnyNode {
        let Node { identity, entry, title, casting, stage, basis } = self;
        Node { identity, entry, title, casting, stage: stage.into(), basis }
    }
}

impl<S> Node<S> {
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
    pub fn casting(&self) -> Casting {
        self.casting
    }

    #[must_use]
    pub fn stage(&self) -> &S {
        &self.stage
    }

    #[must_use]
    pub fn basis(&self) -> Basis {
        self.basis
    }

    fn staged<T>(self, stage: T) -> Node<T> {
        let Node { identity, entry, title, casting, .. } = self;
        Node { identity, entry, title, casting, stage, basis: Basis::Set }
    }
}
