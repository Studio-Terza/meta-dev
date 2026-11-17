mod anchor;
mod binding;
mod boundary;
mod entries;
mod items;
mod port_macro;
mod resolver;
mod resolving;
mod scope;
mod settling;
mod std_names;
mod tables;
mod use_path;
mod view;

use std::collections::BTreeMap;

use graph::{Building, Unresolved};
use lint_layout::{Dependencies, Name};

use anchor::{Anchor, Anchors};
use binding::Binding;
pub(crate) use boundary::{ScopeId, ScopeKey, ScopeMap};
pub(crate) use entries::{Declared, DeclaredKind, Entries, Exposing, Visibility};
pub(crate) use graph::Domain;
pub(crate) use items::ItemIds;
pub(crate) use port_macro::PortMacro;
pub(crate) use resolver::Resolver;
pub(crate) use resolving::Resolving;
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
                entries: Entries::declaring(Declared::new(
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
        let mut chunks = ops.into_iter().map(|op| op.chunk(&scopes)).fold(BTreeMap::new(), |chunked, chunk| {
            let mut grown: BTreeMap<ScopeId, Vec<Entries>> = chunked;
            grown.entry(chunk.scope).or_default().push(chunk.entries);
            grown
        });
        let top_entries = [top].into_iter().chain(chunks.remove(&ScopeId::FIRST).into_iter().flatten()).collect();
        let nested = shapes
            .into_iter()
            .zip(ScopeId::FIRST.following())
            .map(|(shape, id)| Opening { entries: chunks.remove(&id).into_iter().flatten().collect(), ..shape })
            .collect();
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

#[derive(Debug, Default)]
pub(crate) struct Namespace<'read> {
    scopes: Vec<ScopeRow>,
    crates: BTreeMap<&'read Name, ScopeId>,
    dependencies: BTreeMap<ScopeId, &'read Dependencies>,
}

impl<'read> Namespace<'read> {
    pub(crate) fn placed(&mut self, top: Top<'read>, openings: Openings) -> ScopeId {
        let base = ScopeId::after(&self.scopes);
        self.topped(top, openings.top.above(base));
        openings.nested.into_iter().fold(base, |placed, opening| {
            self.opened(opening.parent.above(placed), opening.kind, opening.entries.above(placed));
            placed
        })
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

    pub(super) fn expanded(&self, viewer: ScopeId) -> Unresolved {
        self.chain(viewer)
            .iter()
            .filter_map(|scope| scope.row(&self.scopes))
            .map(|row| row.entries().unresolved())
            .max()
            .unwrap_or(Unresolved::Unbound)
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
