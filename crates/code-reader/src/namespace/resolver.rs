use std::iter;

use lint_layout::Name;

use super::Namespace;
use super::anchor::Anchor;
use super::binding::Binding;
use super::boundary::{GlobId, Lattice, ScopeId};
use super::entries::Sight;
use super::settling::Target;
use super::std_names::{Canonical, StdNames, Table};
use super::tables::{Domain, Key, Lookup, Tables};
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
            Self::Known(binding) | Self::Assumed(binding) => binding,
        }
    }

    pub(super) fn known_or(known: Option<Binding>, assumed: impl FnOnce() -> Binding) -> Self {
        match known {
            Some(binding) => Self::Known(binding),
            None => Self::Assumed(assumed()),
        }
    }

    pub(super) fn known(self) -> Resolution {
        match self {
            Self::Known(binding) => Resolution::Bound(binding),
            Self::Assumed(_) => Resolution::Waiting,
        }
    }
}

impl From<Option<&Canonical>> for Outer {
    fn from(canonical: Option<&Canonical>) -> Self {
        match canonical {
            Some(found) => Self::Known(Binding::Std(found.path())),
            None => Self::Assumed(Binding::Unknown),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Resolution {
    Waiting,
    Bound(Binding),
}

impl Resolution {
    pub(super) fn forced(self) -> Binding {
        match self {
            Self::Waiting => Binding::Unknown,
            Self::Bound(binding) => binding,
        }
    }

    fn kept(self, held: &Binding) -> Self {
        match self {
            Self::Waiting => Self::Bound(held.clone()),
            Self::Bound(found) => Self::Bound(held.agreed(found)),
        }
    }

    fn target(self, sight_of: impl FnOnce(ScopeId) -> Sight) -> Target {
        match self {
            Self::Waiting => Target::Waiting,
            Self::Bound(binding) => binding.target(sight_of),
        }
    }
}

impl Lattice for Resolution {
    fn joined(&self, computed: Self) -> Self {
        match self {
            Self::Waiting => computed,
            Self::Bound(held) => computed.kept(held),
        }
    }
}

impl<'rows> From<&'rows Resolution> for View<'rows> {
    fn from(resolution: &'rows Resolution) -> Self {
        match resolution {
            Resolution::Waiting => View::WAITING,
            Resolution::Bound(binding) => View::from(binding),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Resolver<'tables, 'rows> {
    space: &'tables Namespace<'tables>,
    tables: &'tables Tables<'rows>,
    ignored: Option<GlobId>,
}

impl<'tables, 'rows> Resolver<'tables, 'rows> {
    pub(crate) const fn new(space: &'tables Namespace<'tables>, tables: &'tables Tables<'rows>) -> Self {
        Resolver { space, tables, ignored: None }
    }

    pub(super) fn path(&self, path: &UsePath, viewer: ScopeId) -> Resolution {
        path.tail()
            .iter()
            .fold(path.head().started(self, viewer), |resolution, segment| self.stepped(resolution, segment, viewer))
    }

    pub(super) fn glob_target(&self, path: &UsePath, scope: ScopeId, glob: GlobId) -> Target {
        let resolver = Resolver { space: self.space, tables: self.tables, ignored: Some(glob) };
        resolver.path(path, scope).target(|module| Sight::ByGlob.beyond(self.space.within_reach(module, scope)))
    }

    pub(super) fn global(&self, first: &Name) -> Resolution {
        Resolution::Bound(self.space.global(first))
    }

    pub(super) fn anchored(&self, anchor: Anchor, scope: ScopeId) -> Resolution {
        Resolution::Bound(Binding::from(anchor.anchored(self.space.anchors(scope))))
    }

    pub(super) fn macro_named(&self, first: &Name, viewer: ScopeId) -> Resolution {
        self.chained(viewer, Key::new(Domain::Macros, first), Reading::Written)
            .settled(|| Outer::from(StdNames::canonical(Table::Derives, first)))
    }

    pub(super) fn named(&self, first: &Name, shape: Shape, reading: Reading, viewer: ScopeId) -> Resolution {
        self.chained(viewer, Key::new(Domain::Types, first), reading)
            .settled(|| shape.fallback(first, reading, self.space.prelude(first, viewer), self.space))
    }

    fn entered(&self, module: ScopeId, segment: &Name, viewer: ScopeId) -> Resolution {
        match Anchor::spelled(segment) {
            Some(anchor) => self.anchored(anchor, module),
            None => self
                .tables
                .found(
                    module,
                    Key::new(Domain::Types, segment),
                    Lookup::new(Sight::ByPath.beyond(self.space.within_reach(module, viewer)), None, Reading::Written),
                )
                .settled(|| Outer::Assumed(Binding::Unknown)),
        }
    }

    fn stepped(&self, resolution: Resolution, segment: &Name, viewer: ScopeId) -> Resolution {
        match resolution {
            Resolution::Waiting => Resolution::Waiting,
            Resolution::Bound(binding) => binding.stepped(segment, |module| self.entered(module, segment, viewer)),
        }
    }

    fn chained(&self, scope: ScopeId, key: Key<'_>, reading: Reading) -> View<'rows> {
        let levels = self.space.chain(scope);
        let first = |reading: Reading| Lookup::new(Sight::Inside, self.ignored, reading);
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
