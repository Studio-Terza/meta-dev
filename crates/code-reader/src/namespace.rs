mod anchor;
mod binding;
mod boundary;
mod entries;
mod items;
mod resolver;
mod resolving;
mod scope;
mod settling;
mod std_names;
mod tables;
mod use_path;
mod view;

use std::collections::BTreeMap;

use code_model::{Building, ItemPath};
use lint_layout::Name;

use anchor::{Anchor, Anchors};
use binding::Binding;
pub(crate) use boundary::ScopeId;
pub(crate) use entries::{Declared, DeclaredKind, Entries, Visibility};
pub(crate) use items::ItemIds;
pub(crate) use resolver::Resolver;
pub(crate) use resolving::Resolving;
use scope::ScopeRow;
pub(crate) use scope::{CrateForm, ScopeKind};
pub(crate) use settling::Seeds;
pub(crate) use std_names::StdNames;
pub(crate) use tables::Domain;
pub(crate) use use_path::UsePath;

#[derive(Debug)]
pub(crate) struct Export {
    root: ScopeId,
    path: UsePath,
}

impl Export {
    pub(crate) fn recorded(&self, resolving: &Resolving<'_, '_, '_>, mut building: Building) -> Building {
        building.export(self.path.written(), &resolving.path(self.root, &self.path, Domain::Types));
        building
    }
}

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
        let child = parent.row(&self.scopes).map(|row| kind.opened(row, scope, entries));
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

    pub(crate) fn exports(&self) -> Vec<Export> {
        self.crates
            .iter()
            .filter_map(|(krate, root)| Some((*krate, *root, root.row(&self.scopes)?)))
            .flat_map(|(krate, root, row)| {
                row.entries().exported().map(move |entry| Export { root, path: UsePath::exported(krate, entry) })
            })
            .collect()
    }

    pub(crate) fn issued(&self, building: &mut Building) -> ItemIds {
        self.scopes.iter().map(|row| row.issued(building)).collect()
    }

    pub(super) fn anchors(&self, scope: ScopeId) -> Anchors {
        scope.row(&self.scopes).map(ScopeRow::anchors).unwrap_or_default()
    }

    pub(super) fn chain(&self, scope: ScopeId) -> &[ScopeId] {
        scope.row(&self.scopes).map(ScopeRow::chain).unwrap_or_default()
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
