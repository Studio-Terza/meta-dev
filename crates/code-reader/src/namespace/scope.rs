use std::iter;

use code_model::ItemPath;
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
    fn anchors(self, root: ScopeId) -> Anchors {
        match self {
            CrateForm::Planted => Anchors::rooted(root, Some(root)),
            CrateForm::Island => Anchors::rooted(root, None),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ScopeKind {
    Module(Name),
    Block,
}

impl ScopeKind {
    pub(super) fn opened(self, parent: &ScopeRow, scope: ScopeId, entries: Entries) -> ScopeRow {
        match self {
            ScopeKind::Module(name) => ScopeRow {
                anchors: parent.anchors.nested(scope),
                chain: vec![scope],
                ancestry: iter::once(scope).chain(parent.ancestry.iter().copied()).collect(),
                located: parent.located.as_ref().iter().cloned().chain([name]).collect(),
                entries,
            },
            ScopeKind::Block => ScopeRow {
                anchors: parent.anchors,
                chain: iter::once(scope).chain(parent.chain.iter().copied()).collect(),
                ancestry: iter::once(scope).chain(parent.ancestry.iter().copied()).collect(),
                located: parent.located.clone(),
                entries,
            },
        }
    }
}

#[derive(Debug)]
pub(super) struct ScopeRow {
    anchors: Anchors,
    chain: Vec<ScopeId>,
    ancestry: Vec<ScopeId>,
    located: ItemPath,
    entries: Entries,
}

impl ScopeRow {
    pub(super) fn root(scope: ScopeId, form: CrateForm, entries: Entries) -> ScopeRow {
        ScopeRow {
            anchors: form.anchors(scope),
            chain: vec![scope],
            ancestry: vec![scope],
            located: ItemPath::from_iter([Name::from(Anchor::CRATE)]),
            entries,
        }
    }

    pub(super) fn anchors(&self) -> Anchors {
        self.anchors
    }

    pub(super) fn chain(&self) -> &[ScopeId] {
        &self.chain
    }

    pub(super) fn located(&self) -> &ItemPath {
        &self.located
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
