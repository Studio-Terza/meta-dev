use std::iter;

use code_model::ItemPath;
use lint_layout::Name;

use super::Namespace;
use super::anchor::{Anchor, Anchors};
use super::binding::Locating;
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
            CrateForm::Planted => Anchors::rooted(root),
            CrateForm::Island => Anchors::default(),
        }
    }

    fn located(self) -> Option<ItemPath> {
        match self {
            CrateForm::Planted => Some(ItemPath::default().joined(&Name::from(Anchor::CRATE))),
            CrateForm::Island => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScopeKind {
    Module,
    Block,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Place {
    Root(CrateForm),
    Within(ScopeKind, ScopeId),
}

impl Place {
    pub(super) fn located(self, space: &Namespace, scope: ScopeId) -> Option<ItemPath> {
        match self {
            Place::Root(form) => form.located(),
            Place::Within(kind, above) => kind.located(space, above, scope),
        }
    }
}

impl ScopeKind {
    fn located(self, space: &Namespace, above: ScopeId, scope: ScopeId) -> Option<ItemPath> {
        match self {
            ScopeKind::Module => space.within(above, scope),
            ScopeKind::Block => space.located(above),
        }
    }

    pub(super) fn opened(self, parent: &ScopeRow, above: ScopeId, scope: ScopeId, entries: Entries) -> ScopeRow {
        ScopeRow {
            anchors: self.anchors(parent.anchors, scope),
            chain: self.chain(&parent.chain, scope),
            ancestry: iter::once(scope).chain(parent.ancestry.iter().copied()).collect(),
            place: Place::Within(self, above),
            entries,
        }
    }

    fn anchors(self, parent: Anchors, scope: ScopeId) -> Anchors {
        match self {
            ScopeKind::Module => parent.nested(scope),
            ScopeKind::Block => parent,
        }
    }

    fn chain(self, parent: &[ScopeId], scope: ScopeId) -> Vec<ScopeId> {
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
    place: Place,
    entries: Entries,
}

impl ScopeRow {
    pub(super) fn root(scope: ScopeId, form: CrateForm, entries: Entries) -> ScopeRow {
        ScopeRow {
            anchors: form.anchors(scope),
            chain: vec![scope],
            ancestry: vec![scope],
            place: Place::Root(form),
            entries,
        }
    }

    pub(super) fn anchors(&self) -> Anchors {
        self.anchors
    }

    pub(super) fn chain(&self) -> &[ScopeId] {
        &self.chain
    }

    pub(super) fn place(&self) -> Place {
        self.place
    }

    pub(super) fn entries(&self) -> &Entries {
        &self.entries
    }

    pub(super) fn entries_mut(&mut self) -> &mut Entries {
        &mut self.entries
    }

    pub(super) fn ancestor(&self, level: ScopeId) -> Option<ScopeId> {
        self.ancestry.iter().copied().find(|ancestor| *ancestor == level)
    }
}
