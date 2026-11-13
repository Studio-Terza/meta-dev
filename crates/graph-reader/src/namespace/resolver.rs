use std::iter;

use graph::{Domain, Unresolved};
use lint_layout::Name;

use super::Namespace;
use super::anchor::Anchor;
use super::binding::Binding;
use super::boundary::{GlobId, Lattice, ScopeId};
use super::entries::Sight;
use super::settling::Target;
use super::std_names::{Canonical, StdNames, Table};
use super::tables::{Key, Lookup, Tables};
use super::use_path::{Reading, Shape, UsePath};
use super::view::View;

#[derive(Debug)]
pub(super) enum Outer {
    Known(Binding),
    Assumed(Binding),
}

impl Outer {
    pub(super) fn binding(self) -> Binding {
        match self {
            Outer::Known(binding) | Outer::Assumed(binding) => binding,
        }
    }

    pub(super) fn known_or(known: Option<Binding>, assumed: impl FnOnce() -> Binding) -> Outer {
        match known {
            Some(binding) => Outer::Known(binding),
            None => Outer::Assumed(assumed()),
        }
    }

    pub(super) fn unresolved(self, expanded: impl FnOnce() -> Unresolved) -> Unresolved {
        match self {
            Outer::Known(_) => Unresolved::Unbound,
            Outer::Assumed(_) => expanded(),
        }
    }

    pub(super) fn known(self) -> Resolution {
        match self {
            Outer::Known(binding) => Resolution::Bound(binding),
            Outer::Assumed(_) => Resolution::Waiting,
        }
    }
}

impl From<Option<&Canonical>> for Outer {
    fn from(canonical: Option<&Canonical>) -> Outer {
        match canonical {
            Some(found) => Outer::Known(Binding::Std(found.path())),
            None => Outer::Assumed(Binding::UNBOUND),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Resolution {
    Waiting,
    Bound(Binding),
}

impl Resolution {
    pub(super) fn present(&self) -> Option<&Resolution> {
        match self {
            Resolution::Bound(Binding::Absent) => None,
            Resolution::Waiting | Resolution::Bound(_) => Some(self),
        }
    }

    pub(super) fn forced(self) -> Binding {
        match self {
            Resolution::Waiting => Binding::UNBOUND,
            Resolution::Bound(binding) => binding,
        }
    }

    fn kept(self, held: &Binding) -> Resolution {
        match self {
            Resolution::Waiting => Resolution::Bound(held.clone()),
            Resolution::Bound(found) => Resolution::Bound(held.agreed(found)),
        }
    }

    fn target(self, sight_of: impl FnOnce(ScopeId) -> Sight) -> Target {
        match self {
            Resolution::Waiting => Target::Waiting,
            Resolution::Bound(binding) => binding.target(sight_of),
        }
    }
}

impl Lattice for Resolution {
    fn joined(&self, computed: Resolution) -> Resolution {
        match self {
            Resolution::Waiting => computed,
            Resolution::Bound(held) => computed.kept(held),
        }
    }
}

impl<'rows> From<&'rows Resolution> for View<'rows> {
    fn from(resolution: &'rows Resolution) -> View<'rows> {
        match resolution {
            Resolution::Waiting => View::WAITING,
            Resolution::Bound(binding) => View::from(binding),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Importing<'tables>(Option<Key<'tables>>);

impl Importing<'_> {
    fn levels<'levels>(self, levels: &'levels [ScopeId], key: Key<'_>) -> &'levels [ScopeId] {
        match self.0.filter(|importing| *importing == key) {
            Some(_) => levels.get(1..).unwrap_or_default(),
            None => levels,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Resolver<'tables, 'rows> {
    space: &'tables Namespace<'tables>,
    tables: &'tables Tables<'rows>,
    ignored: Option<GlobId>,
    importing: Option<Key<'tables>>,
}

impl<'tables, 'rows> Resolver<'tables, 'rows> {
    pub(crate) fn new(space: &'tables Namespace<'tables>, tables: &'tables Tables<'rows>) -> Resolver<'tables, 'rows> {
        Resolver { space, tables, ignored: None, importing: None }
    }

    pub(super) fn importing(self, key: Key<'tables>) -> Resolver<'tables, 'rows> {
        Resolver { space: self.space, tables: self.tables, ignored: self.ignored, importing: Some(key) }
    }

    pub(super) fn path(&self, path: &UsePath, viewer: ScopeId, domain: Domain) -> Resolution {
        let tail = path.tail();
        let (within, last) = tail.split_at(tail.len().saturating_sub(1));
        let module = within.iter().fold(path.head().started(self, viewer, domain), |resolution, segment| {
            self.stepped(resolution, segment, viewer, Domain::Types)
        });
        last.iter().fold(module, |resolution, segment| self.stepped(resolution, segment, viewer, domain))
    }

    pub(super) fn glob_target(&self, path: &UsePath, scope: ScopeId, glob: GlobId) -> Target {
        let resolver = Resolver { space: self.space, tables: self.tables, ignored: Some(glob), importing: None };
        resolver
            .path(path, scope, Domain::Types)
            .target(|module| Sight::ByGlob.beyond(self.space.within_reach(module, scope)))
    }

    pub(super) fn space(&self) -> &'tables Namespace<'tables> {
        self.space
    }

    pub(super) fn anchored(&self, anchor: Anchor, scope: ScopeId) -> Resolution {
        Resolution::Bound(Binding::from(anchor.anchored(self.space.anchors(scope))))
    }

    pub(super) fn macro_named(&self, first: &Name, viewer: ScopeId) -> Resolution {
        let derive = StdNames::canonical(Table::Derives, first);
        self.chained(viewer, Key::new(Domain::Macros, first), Reading::Written).contested(
            derive.map(|found| Binding::Std(found.path())),
            || Outer::from(derive),
            || self.space.expanded(viewer),
        )
    }

    pub(super) fn keyed(&self, key: Key<'_>, viewer: ScopeId) -> Resolution {
        self.chained(viewer, key, Reading::Written).contested(
            None,
            || Outer::Assumed(Binding::UNBOUND),
            || self.space.expanded(viewer),
        )
    }

    pub(super) fn named(&self, first: &Name, shape: Shape, reading: Reading, viewer: ScopeId) -> Resolution {
        self.chained(viewer, Key::new(Domain::Types, first), reading).contested(
            shape.primitive(first).map(Binding::Primitive),
            || shape.fallback(first, reading, self.space, viewer),
            || self.space.expanded(viewer),
        )
    }

    fn entered(&self, module: ScopeId, segment: &Name, viewer: ScopeId, domain: Domain) -> Resolution {
        match Anchor::spelled(segment) {
            Some(anchor) => self.anchored(anchor, module),
            None => self
                .tables
                .found(
                    module,
                    Key::new(domain, segment),
                    Lookup::new(Sight::ByPath.beyond(self.space.within_reach(module, viewer)), None, Reading::Written),
                )
                .settled(|| Outer::Assumed(Binding::Absent), || self.space.expanded(module)),
        }
    }

    fn stepped(&self, resolution: Resolution, segment: &Name, viewer: ScopeId, domain: Domain) -> Resolution {
        match resolution {
            Resolution::Waiting => Resolution::Waiting,
            Resolution::Bound(binding) => {
                binding.stepped(segment, |module| self.entered(module, segment, viewer, domain))
            }
        }
    }

    fn chained(&self, scope: ScopeId, key: Key<'_>, reading: Reading) -> View<'rows> {
        let levels = Importing(self.importing).levels(self.space.chain(scope), key);
        let first = |asked: Reading| Lookup::new(Sight::Inside, self.ignored, asked);
        self.walked(levels, key, first(reading)).disputed(
            || self.walked(levels, key, first(Reading::Explicit)),
            || self.walked(levels, key, first(Reading::Imported)),
        )
    }

    fn walked(&self, levels: &[ScopeId], key: Key<'_>, first: Lookup) -> View<'rows> {
        let lookups = iter::once(first).chain(iter::repeat(first.outer()));
        levels
            .iter()
            .zip(lookups)
            .fold(View::ABSENT, |found, (level, lookup)| found.or_else(|| self.tables.found(*level, key, lookup)))
    }
}
