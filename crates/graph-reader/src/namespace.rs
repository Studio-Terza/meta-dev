mod anchor;
mod binding;
mod boundary;
mod entries;
mod items;
mod port_macro;
mod resolver;
mod resolving;
mod rules;
mod scope;
mod settling;
mod std_names;
mod tables;
mod use_path;
mod view;

use std::collections::BTreeMap;

use graph::Building;
use lint_layout::{Dependencies, Name};

use anchor::{Anchor, Anchors};
use binding::Binding;
pub(crate) use boundary::{KeyRole, NodeKey, ScopeId, ScopeKey, ScopeMap};
use entries::DeclaredAt;
pub(crate) use entries::{Declared, DeclaredKind, Entries, Exposing, Visibility};
pub(crate) use graph::Domain;
pub(crate) use items::ItemIds;
pub(crate) use port_macro::PortMacro;
pub(crate) use resolver::Resolver;
pub(crate) use resolving::{Invoked, Invoker, Resolving, TranscribedRow, Transcriptions};
use rules::RulesDefinition;
pub(crate) use rules::TokenNames;
use scope::ScopeRow;
pub(crate) use scope::{CrateForm, ScopeKind};
pub(crate) use settling::Seeds;
pub(crate) use std_names::StdNames;
pub(crate) use use_path::UsePath;

#[derive(Debug)]
pub(crate) struct Export {
    root: ScopeId,
    path: UsePath,
    domain: Domain,
}

impl Export {
    pub(crate) fn recorded(&self, resolving: &Resolving<'_, '_, '_>, mut building: Building) -> Building {
        building.export(self.domain, self.path.written(), &resolving.path(self.root, &self.path, self.domain));
        building
    }
}

#[derive(Debug)]
struct Opening {
    parent: ScopeId,
    kind: ScopeKind,
    entries: Entries,
}

#[derive(Debug)]
pub(crate) struct Openings {
    top: Entries,
    nested: Vec<Opening>,
}

#[derive(Debug)]
pub(crate) enum ScopeOp {
    Opened { key: ScopeKey, parent: ScopeKey, kind: ScopeKind, entries: Entries },
    Absorbed { key: ScopeKey, entries: Entries },
    Declared { key: ScopeKey, name: Name, visibility: Visibility, module: ScopeKey },
}

impl ScopeOp {
    fn opened(&self) -> Option<&ScopeKey> {
        match self {
            ScopeOp::Opened { key, .. } => Some(key),
            ScopeOp::Absorbed { .. } | ScopeOp::Declared { .. } => None,
        }
    }

    fn shaped(&self, scopes: &ScopeMap) -> Option<Opening> {
        match self {
            ScopeOp::Opened { parent, kind, .. } => {
                Some(Opening { parent: scopes.scope(parent), kind: *kind, entries: Entries::default() })
            }
            ScopeOp::Absorbed { .. } | ScopeOp::Declared { .. } => None,
        }
    }

    fn chunk(self, scopes: &ScopeMap) -> Chunk {
        match self {
            ScopeOp::Opened { key, entries, .. } | ScopeOp::Absorbed { key, entries } => {
                Chunk { scope: scopes.scope(&key), entries }
            }
            ScopeOp::Declared { key, name, visibility, module } => Chunk {
                scope: scopes.scope(&key),
                entries: Entries::of_declared(Declared::new(
                    name,
                    visibility,
                    DeclaredKind::Module(scopes.scope(&module)),
                )),
            },
        }
    }
}

#[derive(Debug)]
pub(crate) struct Replayed {
    openings: Openings,
    scopes: ScopeMap,
}
impl Replayed {
    pub(crate) fn concluded<Out>(self, into: impl FnOnce(Openings, ScopeMap) -> Out) -> Out {
        into(self.openings, self.scopes)
    }
}

impl Openings {
    pub(crate) fn replayed(top: Entries, ops: Vec<ScopeOp>) -> Replayed {
        let scopes = ScopeMap::keyed(ops.iter().filter_map(ScopeOp::opened).cloned());
        let shapes: Vec<Opening> = ops.iter().filter_map(|op| op.shaped(&scopes)).collect();
        let seeded: Vec<Vec<Entries>> = std::iter::repeat_with(Vec::new).take(shapes.len() + 1).collect();
        let chunked = ops.into_iter().map(|op| op.chunk(&scopes)).fold(seeded, |chunked, chunk| {
            let mut grown = chunked;
            chunk
                .scope
                .row_mut(&mut grown)
                .into_iter()
                .zip([chunk.entries])
                .for_each(|(held, entries)| held.push(entries));
            grown
        });
        let mut merged: Vec<Entries> = chunked.into_iter().map(|chunks| chunks.into_iter().collect()).collect();
        let nested_entries = merged.split_off(1);
        let top_entries = [top].into_iter().chain(merged).collect();
        let nested =
            shapes.into_iter().zip(nested_entries).map(|(shape, entries)| Opening { entries, ..shape }).collect();
        Replayed { openings: Openings { top: top_entries, nested }, scopes }
    }
}

#[derive(Debug)]
struct Chunk {
    scope: ScopeId,
    entries: Entries,
}

#[derive(Debug)]
pub(crate) enum Top<'read> {
    Root { form: CrateForm, name: Option<&'read Name>, dependencies: &'read Dependencies },
    Child { parent: ScopeId, name: Name, visibility: Visibility },
}

impl Top<'_> {
    fn root(&self, base: ScopeId, space: &Namespace<'_>) -> Option<ScopeId> {
        match self {
            Top::Root { .. } => Some(base),
            Top::Child { parent, .. } => Anchor::Crate.anchored(space.anchors(*parent)),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Namespace<'read> {
    scopes: Vec<ScopeRow>,
    crates: BTreeMap<&'read Name, ScopeId>,
    dependencies: BTreeMap<ScopeId, &'read Dependencies>,
}

impl<'read> Namespace<'read> {
    pub(crate) fn placed(&mut self, top: Top<'read>, openings: Openings) -> ScopeId {
        let base = ScopeId::after(&self.scopes);
        let root = top.root(base, self);

        self.topped(top, openings.top.above(base));
        let placed = openings.nested.into_iter().fold(base, |placed, opening| {
            self.opened(opening.parent.above(placed), opening.kind, opening.entries.above(placed));
            placed
        });
        let exported = self.exported_aliases(base);
        let rooted = exported.into_iter().filter_map(|declared| Some(declared).zip(root));
        rooted.for_each(|(declared, scope)| self.declare(scope, declared));
        placed
    }

    pub(super) fn exported_aliases(&self, base: ScopeId) -> Vec<Declared> {
        let placed = ScopeId::ids(&self.scopes).zip(&self.scopes).filter(|(scope, _)| *scope >= base);
        placed.flat_map(|(scope, row)| row.entries().exported_aliases(scope)).collect()
    }

    fn topped(&mut self, top: Top<'read>, entries: Entries) {
        match top {
            Top::Root { form, name, dependencies } => self.planted(form, name, dependencies, entries),
            Top::Child { parent, name, visibility } => self.child_opened(parent, name, visibility, entries),
        }
    }

    fn child_opened(&mut self, parent: ScopeId, name: Name, visibility: Visibility, entries: Entries) {
        let scope = self.opened(parent, ScopeKind::Module, entries);
        self.declare(parent, Declared::new(name, visibility, DeclaredKind::Module(scope)));
    }

    fn planted(
        &mut self,
        form: CrateForm,
        name: Option<&'read Name>,
        dependencies: &'read Dependencies,
        entries: Entries,
    ) {
        let root = ScopeId::after(&self.scopes);
        self.scopes.push(ScopeRow::root(root, form, entries));
        self.crates.extend(name.map(|crate_name| (crate_name, root)));
        self.dependencies.insert(root, dependencies);
    }

    fn opened(&mut self, parent: ScopeId, kind: ScopeKind, entries: Entries) -> ScopeId {
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
            .for_each(|(entries, item)| entries.declare(item));
    }

    pub(crate) fn exports(&self) -> Vec<Export> {
        self.crates
            .iter()
            .filter_map(|(krate, root)| Some((*krate, *root, root.row(&self.scopes)?)))
            .flat_map(|(krate, root, row)| {
                row.entries().exported().map(move |key| Export {
                    root,
                    path: UsePath::exported(krate, key.name()),
                    domain: key.domain(),
                })
            })
            .collect()
    }

    pub(crate) fn issued(&self, building: &mut Building) -> ItemIds {
        let roots: BTreeMap<ScopeId, &Name> = self.crates.iter().map(|(name, root)| (*root, *name)).collect();
        self.scopes
            .iter()
            .map(|row| row.issued(building, row.outermost().and_then(|root| roots.get(&root).copied())))
            .collect()
    }

    pub(super) fn anchors(&self, scope: ScopeId) -> Anchors {
        scope.row(&self.scopes).map_or_default(ScopeRow::anchors)
    }

    pub(super) fn chain(&self, scope: ScopeId) -> &[ScopeId] {
        scope.row(&self.scopes).map_or_default(ScopeRow::chain)
    }

    pub(super) fn prelude(&self, name: &Name, viewer: ScopeId) -> Option<Binding> {
        Anchor::Crate
            .anchored(self.anchors(viewer))
            .and_then(|root| root.row(&self.scopes).and_then(|row| row.entries().prelude(name, self, root)))
    }

    pub(super) fn rules(&self, at: DeclaredAt) -> Option<RulesDefinition> {
        at.scope().row(&self.scopes).and_then(|row| row.entries().rules(at.declared_id()))
    }

    pub(super) fn within_reach(&self, level: ScopeId, from: ScopeId) -> Option<ScopeId> {
        from.row(&self.scopes).and_then(|row| row.ancestor(level))
    }

    pub(super) fn workspace(&self, name: &Name) -> Option<ScopeId> {
        self.crates.get(name).copied()
    }

    pub(super) fn global(&self, first: &Name, dependencies: Option<&Dependencies>) -> Binding {
        StdNames::root(first)
            .or_else(|| self.workspace(first).map(Binding::Module))
            .unwrap_or_else(|| Binding::external(first, dependencies))
    }

    pub(super) fn dependencies(&self, viewer: ScopeId) -> Option<&'read Dependencies> {
        viewer.row(&self.scopes).and_then(ScopeRow::outermost).and_then(|root| self.dependencies.get(&root)).copied()
    }
}
