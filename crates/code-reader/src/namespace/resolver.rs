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
            Outer::Known(binding) | Outer::Assumed(binding) => binding,
        }
    }

    pub(super) fn known_or(known: Option<Binding>, assumed: impl FnOnce() -> Binding) -> Outer {
        match known {
            Some(binding) => Outer::Known(binding),
            None => Outer::Assumed(assumed()),
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
            None => Outer::Assumed(Binding::Unknown),
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
            Resolution::Waiting => Binding::Unknown,
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
pub(crate) struct Resolver<'tables, 'rows> {
    space: &'tables Namespace<'tables>,
    tables: &'tables Tables<'rows>,
    ignored: Option<GlobId>,
}

impl<'tables, 'rows> Resolver<'tables, 'rows> {
    pub(crate) fn new(space: &'tables Namespace<'tables>, tables: &'tables Tables<'rows>) -> Resolver<'tables, 'rows> {
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
        let ignored = iter::once(self.ignored).chain(iter::repeat(None));
        self.space.chain(scope).iter().zip(ignored).fold(View::ABSENT, |found, (level, ignored)| {
            found.or_else(|| self.tables.found(*level, key, Lookup::new(Sight::Inside, ignored, reading)))
        })
    }
}
