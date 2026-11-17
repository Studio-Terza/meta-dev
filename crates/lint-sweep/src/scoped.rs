use std::fmt;

use lint_core::{Address, Descend, Violation};

use crate::Watching;
use crate::hooks::{Kind, Watch};

use crate::{Finds, Gathers};

#[derive(Debug)]
pub struct Attachment<Group> {
    root: Address,
    group: Group,
}

impl<Group> Attachment<Group> {
    pub fn reaching<Node>(node: &Node, group: Group) -> Attachment<Group> {
        Attachment { root: Address::from(node), group }
    }

    pub(crate) fn regrouped<Other>(self, regroup: impl FnOnce(Group) -> Other) -> Attachment<Other> {
        Attachment { root: self.root, group: regroup(self.group) }
    }
}

pub trait Attaches<Group> {
    const ATTACHED: &'static [Kind];

    fn attaching(&mut self) -> Vec<Attachment<Group>>;

    fn detached(&mut self, group: Group);
}

#[derive(Debug, Default)]
struct Listener {
    muted: Option<Address>,
    said: Option<Descend>,
}

impl Listener {
    fn heard<'ast>(
        &mut self,
        watcher: &mut dyn Watch<'ast>,
        kind: Kind,
        call: &mut dyn FnMut(&mut dyn Watch<'ast>) -> Descend,
    ) {
        let listening = self.muted.map_or(Some(()), |_| None).filter(|()| watcher.kinds().contains(&kind));
        self.said = listening.map(|()| call(watcher));
    }

    fn expands(&self) -> Option<()> {
        self.said.filter(|descend| matches!(descend, Descend::Expand | Descend::Unfold)).map(|_| ())
    }

    fn settled(&mut self, at: Address, expanding: Option<()>) {
        let silenced = self.said.take().and_then(|descend| Silence::from(descend).within(expanding));
        self.muted = self.muted.or(silenced.map(|()| at));
    }

    fn left<'ast>(
        &mut self,
        watcher: &mut dyn Watch<'ast>,
        kind: Kind,
        at: Address,
        call: &mut dyn FnMut(&mut dyn Watch<'ast>),
    ) {
        let reached = self.muted.map_or(Some(()), |muted| (muted == at).then_some(()));
        reached.filter(|()| watcher.kinds().contains(&kind)).into_iter().for_each(|()| call(watcher));
        self.muted = self.muted.filter(|muted| *muted != at);
    }
}

#[derive(Clone, Copy, Debug)]
enum Silence {
    Always,
    UnlessExpanding,
    WhenExpanding,
    Never,
}

impl From<Descend> for Silence {
    fn from(descend: Descend) -> Silence {
        match descend {
            Descend::Over => Silence::Always,
            Descend::Expand => Silence::UnlessExpanding,
            Descend::Unfold => Silence::Never,
            Descend::Into => Silence::WhenExpanding,
        }
    }
}

impl Silence {
    fn within(self, expanding: Option<()>) -> Option<()> {
        match self {
            Silence::Always => Some(()),
            Silence::UnlessExpanding => expanding.map_or(Some(()), |()| None),
            Silence::WhenExpanding => expanding,
            Silence::Never => None,
        }
    }
}

#[derive(Debug)]
struct Open<Group> {
    root: Address,
    group: Group,
    heard: Vec<Kind>,
    listeners: Vec<Listener>,
}

impl<Group: Gathers<Group>> From<Attachment<Group>> for Open<Group> {
    fn from(attachment: Attachment<Group>) -> Open<Group> {
        let mut group = attachment.group;
        let watching: Vec<&mut Watching<'_>> = group.watching().into_iter().collect();
        let listeners = watching.iter().map(|_| Listener::default()).collect();
        let mut heard: Vec<Kind> = watching.iter().flat_map(|member| member.kinds().iter().copied()).collect();
        heard.sort();
        heard.dedup();
        Open { root: attachment.root, group, heard, listeners }
    }
}

impl<Group: Gathers<Group>> Open<Group> {
    fn heard<'ast>(&mut self, kind: Kind, call: &mut dyn FnMut(&mut dyn Watch<'ast>) -> Descend) {
        self.group
            .watching()
            .into_iter()
            .zip(self.listeners.iter_mut())
            .for_each(|(member, listener)| listener.heard(member, kind, call));
    }

    fn expands(&self) -> Option<()> {
        self.listeners.iter().find_map(Listener::expands)
    }

    fn settled(&mut self, at: Address, expanding: Option<()>) {
        self.listeners.iter_mut().for_each(|listener| listener.settled(at, expanding));
    }

    fn left<'ast>(&mut self, kind: Kind, at: Address, call: &mut dyn FnMut(&mut dyn Watch<'ast>)) {
        Some(())
            .filter(|()| self.heard.contains(&kind))
            .map(|()| self.group.watching())
            .into_iter()
            .flatten()
            .zip(self.listeners.iter_mut())
            .for_each(|(member, listener)| listener.left(member, kind, at, call));
    }
}

#[derive(Debug)]
pub struct Scoped<Parent, Group> {
    parent: Parent,
    kinds: Vec<Kind>,
    listener: Listener,
    pending: Vec<Attachment<Group>>,
    open: Vec<Open<Group>>,
}

impl<Parent: for<'any> Watch<'any> + Attaches<Group>, Group> Scoped<Parent, Group> {
    pub fn new(parent: Parent) -> Scoped<Parent, Group> {
        let mut kinds: Vec<Kind> = parent.kinds().iter().chain(Parent::ATTACHED).copied().collect();
        kinds.sort();
        kinds.dedup();
        Scoped { parent, kinds, listener: Listener::default(), pending: Vec::new(), open: Vec::new() }
    }
}

impl<Parent, Group> Scoped<Parent, Group> {
    pub fn into_parent(self) -> Parent {
        self.parent
    }
}

impl<Parent: for<'any> Watch<'any> + Attaches<Group>, Group: Gathers<Group>> Scoped<Parent, Group> {
    fn opened(&mut self, at: Address) {
        self.pending.extend(self.parent.attaching());
        self.open.extend(self.pending.extract_if(.., |attachment| attachment.root == at).map(Open::from));
    }

    fn entered<'ast>(
        &mut self,
        kind: Kind,
        at: Address,
        call: &mut dyn FnMut(&mut dyn Watch<'ast>) -> Descend,
    ) -> Descend {
        self.listener.heard(&mut self.parent, kind, call);
        self.opened(at);
        self.open.iter_mut().for_each(|open| open.heard(kind, call));
        let wanted = self.listener.expands().or_else(|| self.open.iter().find_map(Open::expands));
        let expanding = wanted.filter(|()| kind == Kind::Macro);
        self.listener.settled(at, expanding);
        self.open.iter_mut().for_each(|open| open.settled(at, expanding));
        expanding.map_or(Descend::Into, |()| Descend::Unfold)
    }

    fn left<'ast>(&mut self, kind: Kind, at: Address, call: &mut dyn FnMut(&mut dyn Watch<'ast>)) {
        self.listener.left(&mut self.parent, kind, at, call);
        self.open.iter_mut().for_each(|open| open.left(kind, at, call));
        let closed: Vec<Open<Group>> = self.open.extract_if(.., |open| open.root == at).collect();
        closed.into_iter().fold(&mut self.parent, |parent, open| {
            parent.detached(open.group);
            parent
        });
    }
}

impl<'ast, Parent: for<'any> Watch<'any> + Attaches<Group>, Group: Gathers<Group>> Watch<'ast>
    for Scoped<Parent, Group>
{
    scope_nodes!();
}

impl<'file, Parent: Finds<'file> + Attaches<Group>, Group: Gathers<Group> + fmt::Debug> Finds<'file>
    for Scoped<Parent, Group>
{
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        Box::new(self.parent).found()
    }
}
