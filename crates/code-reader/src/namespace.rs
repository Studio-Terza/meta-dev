mod anchor;
mod binding;
mod boundary;
mod converging;
mod entries;
mod resolver;
mod scope;
mod settling;
mod std_names;
mod tables;
mod use_path;
mod view;

use std::collections::BTreeMap;

use code_model::ItemPath;
use lint_layout::Name;

use anchor::{Anchor, Anchors};
use binding::{Binding, Locating};
pub(crate) use boundary::ScopeId;
pub(crate) use entries::{Declared, DeclaredKind, Entries, ItemId, Visibility};
pub(crate) use resolver::Resolver;
use scope::ScopeRow;
pub(crate) use scope::{CrateForm, ScopeKind};
pub(crate) use settling::{Seeds, Settled};
use std_names::StdNames;
pub(crate) use tables::Domain;
pub(crate) use use_path::UsePath;

#[derive(Debug, Default)]
pub(crate) struct Namespace<'read> {
    scopes: Vec<ScopeRow>,
    crates: BTreeMap<&'read Name, ScopeId>,
}

impl<'read> Namespace<'read> {
    pub(crate) fn planted(&mut self, form: CrateForm, name: Option<&'read Name>, entries: Entries) -> ScopeId {
        let root = ScopeId::after(&self.scopes);
        self.scopes.push(ScopeRow::root(root, form, entries));
        self.crates.extend(name.map(|name| (name, root)));
        root
    }

    pub(crate) fn opened(&mut self, parent: ScopeId, kind: ScopeKind, entries: Entries) -> ScopeId {
        let scope = ScopeId::after(&self.scopes);
        let child = parent.row(&self.scopes).map(|row| kind.opened(row, parent, scope, entries));
        self.scopes.extend(child);
        scope
    }

    pub(crate) fn declare(&mut self, scope: ScopeId, declared: Declared) {
        scope
            .row_mut(&mut self.scopes)
            .map(ScopeRow::entries_mut)
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

    pub(super) fn within(&self, above: ScopeId, scope: ScopeId) -> Option<ItemPath> {
        above.row(&self.scopes).and_then(|row| row.entries().within(scope, self.located(above)))
    }

    pub(super) fn prelude(&self, name: &Name, viewer: ScopeId) -> Option<Binding> {
        Anchor::Crate
            .anchored(self.anchors(viewer))
            .and_then(|root| root.row(&self.scopes).and_then(|row| row.entries().prelude(name, self, root)))
    }

    pub(super) fn within_reach(&self, level: ScopeId, from: ScopeId) -> Option<ScopeId> {
        from.row(&self.scopes).and_then(|row| row.ancestor(level))
    }

    pub(super) fn workspace(&self, name: &Name) -> Option<ScopeId> {
        self.crates.get(name).copied()
    }

    pub(super) fn global(&self, first: &Name) -> Binding {
        StdNames::root(first)
            .or_else(|| self.workspace(first).map(Binding::Module))
            .unwrap_or_else(|| Binding::External(ItemPath::default().joined(first)))
    }
}

impl Locating for Namespace<'_> {
    fn located(&self, scope: ScopeId) -> Option<ItemPath> {
        scope.row(&self.scopes).map(ScopeRow::place).and_then(|place| place.located(self, scope))
    }

    fn declared_path(&self, scope: ScopeId, item: ItemId) -> Option<ItemPath> {
        scope.row(&self.scopes).and_then(|row| row.entries().path_of(item, self.located(scope)))
    }
}
