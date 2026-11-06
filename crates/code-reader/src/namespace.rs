mod anchor;
mod binding;
mod boundary;
mod entries;
mod resolver;
mod scope;
mod std_names;
mod use_path;

use std::collections::BTreeMap;

use code_model::ItemPath;
use lint_layout::Name;

use anchor::Anchors;
use binding::{Binding, Globbed};
pub(crate) use boundary::ScopeId;
use entries::Reach;
pub(crate) use entries::{Declared, DeclaredKind, Entries, Visibility};
pub(crate) use resolver::Resolver;
use resolver::Walk;
use scope::ScopeRow;
pub(crate) use scope::{CrateForm, ScopeKind};
use std_names::StdNames;
pub(crate) use use_path::UsePath;

#[derive(Debug, Default)]
pub(crate) struct Namespace {
    scopes: Vec<ScopeRow>,
    crates: BTreeMap<Name, ScopeId>,
}

impl Namespace {
    pub(crate) fn planted(&mut self, form: CrateForm, name: Option<Name>, entries: Entries) -> ScopeId {
        let root = ScopeId::after(&self.scopes);
        self.scopes.push(ScopeRow::root(root, form, entries));
        self.crates.extend(name.map(|name| (name, root)));
        root
    }

    pub(crate) fn opened(&mut self, parent: ScopeId, kind: ScopeKind, entries: Entries) -> ScopeId {
        let scope = ScopeId::after(&self.scopes);
        let child = parent.row(&self.scopes).map(|row| kind.opened(row, scope, entries));
        self.scopes.extend(child);
        scope
    }

    pub(crate) fn declare(&mut self, scope: ScopeId, declared: Declared) {
        scope
            .entries_mut(&mut self.scopes)
            .zip(Some(declared))
            .into_iter()
            .for_each(|(entries, declared)| entries.declare(declared));
    }

    pub(super) fn anchors(&self, scope: ScopeId) -> Anchors {
        scope.row(&self.scopes).map(ScopeRow::anchors).unwrap_or_default()
    }

    pub(super) fn chain(&self, scope: ScopeId) -> &[ScopeId] {
        scope.row(&self.scopes).map(ScopeRow::chain).unwrap_or_default()
    }

    pub(super) fn located(&self, scope: ScopeId) -> ItemPath {
        scope.row(&self.scopes).map(ScopeRow::located).cloned().unwrap_or_default()
    }

    pub(super) fn reach(&self, level: ScopeId, from: ScopeId) -> Reach {
        match from.row(&self.scopes).and_then(|row| row.ancestor(level)) {
            Some(_) => Reach::Inside,
            None => Reach::Outside,
        }
    }

    pub(super) fn level(&self, level: ScopeId, name: &Name, walk: &Walk<'_>) -> Globbed {
        match level.row(&self.scopes) {
            Some(row) => row.entries().found(level, name, walk),
            None => Globbed::Opaque,
        }
    }

    pub(super) fn workspace(&self, name: &Name) -> Option<ScopeId> {
        self.crates.get(name).copied()
    }

    pub(super) fn global(&self, first: &Name) -> Binding {
        StdNames::root(first)
            .or_else(|| self.workspace(first).map(Binding::Module))
            .unwrap_or_else(|| Binding::External(ItemPath::from_iter([first.clone()])))
    }
}
