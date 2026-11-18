use std::iter;

use graph::Item;
use lint_layout::Name;

use super::anchor::{Anchor, Anchors};
use super::boundary::ScopeId;
use super::entries::Entries;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CrateForm {
    Planted,
    Island,
}

impl CrateForm {
    pub(super) fn anchors(self, root: ScopeId) -> Anchors {
        match self {
            CrateForm::Planted => Anchors::rooted(root),
            CrateForm::Island => Anchors::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScopeKind {
    Module,
    Block,
}

impl ScopeKind {
    pub(super) fn anchors(self, parent: Anchors, scope: ScopeId) -> Anchors {
        match self {
            ScopeKind::Module => parent.nested(scope),
            ScopeKind::Block => parent,
        }
    }

    pub(super) fn chain(self, parent: &[ScopeId], scope: ScopeId) -> Vec<ScopeId> {
        match self {
            ScopeKind::Module => vec![scope],
            ScopeKind::Block => iter::once(scope).chain(parent.iter().copied()).collect(),
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
    pub(super) fn new(anchors: Anchors, chain: Vec<ScopeId>, ancestry: Vec<ScopeId>, entries: Entries) -> ScopeRow {
        ScopeRow { anchors, chain, ancestry, entries }
    }

    pub(super) fn anchors(&self) -> Anchors {
        self.anchors
    }

    pub(super) fn chain(&self) -> &[ScopeId] {
        &self.chain
    }

    pub(super) fn entries(&self) -> &Entries {
        &self.entries
    }

    pub(super) fn outermost(&self) -> Option<ScopeId> {
        self.ancestry.last().copied()
    }

    pub(super) fn ancestor(&self, level: ScopeId) -> Option<ScopeId> {
        self.ancestry.iter().copied().find(|ancestor| *ancestor == level)
    }

    pub(super) fn issued(&self, krate: Option<&Name>) -> Vec<Option<Item>> {
        match Anchor::Crate.anchored(self.anchors) {
            Some(_) => self.entries.issued(krate),
            None => Vec::new(),
        }
    }
}
