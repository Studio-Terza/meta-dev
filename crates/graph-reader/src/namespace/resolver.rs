use std::iter;

use graph::{Domain, Unresolved};
use lint_layout::Name;

use super::Namespace;
use super::anchor::Anchor;
use super::binding::Binding;
use algebra::{Monoid, Semigroup};

use super::boundary::{GlobId, ScopeId};
use super::entries::{DeclaredAt, Sight};
use super::rules::{RulesDefinition, Written};
use super::settling::Target;
use super::std_names::{Canonical, StdNames, Table};
use super::tables::{Key, Lookup, Tables};
use super::use_path::{Reading, Shape, UsePath};
use super::view::{Nearest, View};

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

    pub(super) fn known_binding(self) -> Option<Binding> {
        match self {
            Outer::Known(binding) => Some(binding),
            Outer::Assumed(_) => None,
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
            Resolution::Waiting => Some(self),
            Resolution::Bound(binding) => binding.present().map(|_| self),
        }
    }

    pub(super) fn declared(&self) -> Option<DeclaredAt> {
        match self {
            Resolution::Waiting => None,
            Resolution::Bound(binding) => binding.declared(),
        }
    }

    pub(super) fn narrowed(self, rules: Option<RulesDefinition>, written: &Written) -> Target {
        match self {
            Resolution::Waiting => Target::Waiting,
            Resolution::Bound(_) => Target::narrowed(rules, written),
        }
    }

    fn target(self, sight_of: impl FnOnce(ScopeId) -> Sight) -> Target {
        match self {
            Resolution::Waiting => Target::Waiting,
            Resolution::Bound(binding) => binding.target(sight_of),
        }
    }
}

impl Semigroup for Resolution {
    fn combine(self, computed: Resolution) -> Resolution {
        Resolution::from(Option::<Binding>::from(self).combine(Option::from(computed)))
    }
}

impl From<Resolution> for Option<Binding> {
    fn from(resolution: Resolution) -> Option<Binding> {
        match resolution {
            Resolution::Waiting => None,
            Resolution::Bound(binding) => Some(binding),
        }
    }
}

impl From<Option<Binding>> for Resolution {
    fn from(binding: Option<Binding>) -> Resolution {
        binding.map_or(Resolution::Waiting, Resolution::Bound)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Dispute {
    Clear,
    Contested,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Deference {
    Stands,
    Defers,
}

#[derive(Debug)]
struct Walks<'walk, 'tables, 'rows, 'key> {
    resolver: &'walk Resolver<'tables, 'rows>,
    levels: &'walk [ScopeId],
    key: Key<'key>,
    written: View<'rows>,
}

impl<'rows> Walks<'_, '_, 'rows, '_> {
    fn walk(&self, reading: Reading) -> View<'rows> {
        self.resolver.walked(self.levels, self.key, Lookup::new(Sight::Inside, self.resolver.ignored, reading))
    }

    fn settled(&self, dispute: Dispute) -> View<'rows> {
        match dispute {
            Dispute::Clear => self.written,
            Dispute::Contested => self.deferred(self.walk(Reading::Explicit).deference()),
        }
    }

    fn deferred(&self, deference: Deference) -> View<'rows> {
        match deference {
            Deference::Stands => self.written,
            Deference::Defers => self.walk(Reading::Imported),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Importing<'tables>(Option<Key<'tables>>);

impl Importing<'_> {
    fn levels<'levels>(self, levels: &'levels [ScopeId], key: Key<'_>) -> &'levels [ScopeId] {
        match self.0.filter(|importing| *importing == key).filter(|_| key.domain() != Domain::Macros) {
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
            .target(|module| self.space.sight(module, scope).unwrap_or(Sight::ByGlob))
    }

    pub(super) fn invoked_target(&self, path: &UsePath, scope: ScopeId, glob: GlobId, written: &Written) -> Target {
        let resolver = Resolver { space: self.space, tables: self.tables, ignored: Some(glob), importing: None };
        let found = resolver.path(path, scope, Domain::Macros);
        let rules = found.declared().and_then(|at| self.space.rules(at));
        found.narrowed(rules, written)
    }

    pub(super) fn expanded(&self, viewer: ScopeId, name: &Name) -> Unresolved {
        let chain = self.space.chain(viewer).iter();
        chain.map(|scope| self.tables.unresolved(*scope, name)).fold(Unresolved::Unbound, Ord::max)
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
            || derive.map(|found| Binding::Std(found.path())),
            || Outer::from(derive),
            || self.expanded(viewer, first),
        )
    }

    pub(super) fn keyed(&self, key: Key<'_>, viewer: ScopeId) -> Resolution {
        self.chained(viewer, key, Reading::Written).contested(
            || None,
            || Outer::Assumed(Binding::UNBOUND),
            || self.expanded(viewer, key.name()),
        )
    }

    pub(super) fn named(&self, first: &Name, shape: Shape, reading: Reading, viewer: ScopeId) -> Resolution {
        self.chained(viewer, Key::new(Domain::Types, first), reading).contested(
            || shape.builtin(first, reading, self.space, viewer),
            || shape.fallback(first, reading, self.space, viewer),
            || self.expanded(viewer, first),
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
                    Lookup::new(self.space.sight(module, viewer).unwrap_or(Sight::ByPath), None, Reading::Written),
                )
                .settled(|| Outer::Assumed(Binding::Absent), || self.expanded(module, segment)),
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
        let written = self.walked(levels, key, Lookup::new(Sight::Inside, self.ignored, reading));
        Walks { resolver: self, levels, key, written }.settled(written.dispute())
    }

    fn walked(&self, levels: &[ScopeId], key: Key<'_>, first: Lookup) -> View<'rows> {
        let lookups = iter::once(first).chain(iter::repeat(first.outer()));
        let views = levels.iter().zip(lookups).map(|(level, lookup)| self.tables.found(*level, key, lookup));
        View::from(Nearest::concat(views.map(Nearest::from)))
    }
}
