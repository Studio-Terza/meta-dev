use std::iter;

use code_model::{Building, ItemId};

use super::anchor::{Anchor, Anchors};
use super::boundary::ScopeId;
use super::entries::Entries;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CrateForm {
    Planted,
    Island,
}

impl CrateForm {
    fn anchors(self, root: ScopeId) -> Anchors {
        match self {
            Self::Planted => Anchors::rooted(root),
            Self::Island => Anchors::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScopeKind {
    Module,
    Block,
}

impl ScopeKind {
    pub(super) fn opened(self, parent: &ScopeRow, scope: ScopeId, entries: Entries) -> ScopeRow {
        ScopeRow {
            anchors: self.anchors(parent.anchors, scope),
            chain: self.chain(&parent.chain, scope),
            ancestry: iter::once(scope).chain(parent.ancestry.iter().copied()).collect(),
            entries,
        }
    }

    const fn anchors(self, parent: Anchors, scope: ScopeId) -> Anchors {
        match self {
            Self::Module => parent.nested(scope),
            Self::Block => parent,
        }
    }

    fn chain(self, parent: &[ScopeId], scope: ScopeId) -> Vec<ScopeId> {
        match self {
            Self::Module => vec![scope],
            Self::Block => iter::once(scope).chain(parent.iter().copied()).collect(),
        }
    }
}

#[derive(Debug)]
pub(super) struct ScopeRow {
    anchors: Anchors,
    chain: Vec<ScopeId>,
    ancestry: Vec<ScopeId>,
    entries: Entries,
}

impl ScopeRow {
    pub(super) fn root(scope: ScopeId, form: CrateForm, entries: Entries) -> Self {
        Self { anchors: form.anchors(scope), chain: vec![scope], ancestry: vec![scope], entries }
    }

    pub(super) const fn anchors(&self) -> Anchors {
        self.anchors
    }

    pub(super) fn chain(&self) -> &[ScopeId] {
        &self.chain
    }

    pub(super) const fn entries(&self) -> &Entries {
        &self.entries
    }

    pub(super) const fn entries_mut(&mut self) -> &mut Entries {
        &mut self.entries
    }

    pub(super) fn ancestor(&self, level: ScopeId) -> Option<ScopeId> {
        self.ancestry.iter().copied().find(|ancestor| *ancestor == level)
    }

    pub(super) fn issued(&self, building: &mut Building) -> Vec<Option<ItemId>> {
        match Anchor::Crate.anchored(self.anchors) {
            Some(_) => self.entries.issued(building),
            None => Vec::new(),
        }
    }
}
