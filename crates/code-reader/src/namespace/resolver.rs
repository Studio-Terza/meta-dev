use code_model::Resolved;
use lint_layout::Name;

use super::Namespace;
use super::anchor::Anchor;
use super::binding::{Binding, Globbed};
use super::boundary::ScopeId;
use super::entries::Sight;
use super::std_names::{StdNames, Table};
use super::use_path::{Shape, UsePath};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Domain {
    Types,
    Macros,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Step<'trail> {
    scope: ScopeId,
    name: &'trail Name,
    domain: Domain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Expansion<'trail> {
    scope: ScopeId,
    path: &'trail UsePath,
}

#[derive(Clone, Copy, Debug)]
struct Linked<'trail, Item> {
    item: Item,
    previous: &'trail Chain<'trail, Item>,
}

#[derive(Clone, Copy, Debug)]
struct Chain<'trail, Item> {
    link: Option<Linked<'trail, Item>>,
}

impl<'trail, Item: Copy + PartialEq> Chain<'trail, Item> {
    fn items(&self) -> impl Iterator<Item = Item> + '_ {
        std::iter::successors(Some(self), |chain| chain.link.map(|linked| linked.previous))
            .filter_map(|chain| chain.link.map(|linked| linked.item))
    }

    fn pushed(&'trail self, item: Item) -> Chain<'trail, Item> {
        Chain { link: Some(Linked { item, previous: self }) }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Trail<'trail> {
    visited: Chain<'trail, Step<'trail>>,
    expanding: Chain<'trail, Expansion<'trail>>,
}

impl Trail<'static> {
    const EMPTY: Trail<'static> = Trail { visited: Chain { link: None }, expanding: Chain { link: None } };
}

impl<'trail> Step<'trail> {
    const LIMIT: usize = 64;

    fn traced(self, trail: &'trail Trail<'trail>) -> Tracing<'trail> {
        match trail.visited.items().find(|visited| *visited == self).or_else(|| trail.visited.items().nth(Self::LIMIT))
        {
            Some(_) => Tracing::Repeated,
            None => Tracing::Fresh(Trail { visited: trail.visited.pushed(self), expanding: trail.expanding }),
        }
    }
}

impl<'trail> Expansion<'trail> {
    fn traced(self, trail: &'trail Trail<'trail>) -> Tracing<'trail> {
        match trail.expanding.items().find(|expanding| *expanding == self) {
            Some(_) => Tracing::Repeated,
            None => Tracing::Fresh(Trail { visited: Chain { link: None }, expanding: trail.expanding.pushed(self) }),
        }
    }
}

#[derive(Debug)]
pub(super) enum Tracing<'trail> {
    Fresh(Trail<'trail>),
    Repeated,
}

impl Tracing<'_> {
    pub(super) fn globbed(self, walk: &Walk<'_>, path: &UsePath, name: &Name, domain: Domain) -> Globbed {
        match self {
            Tracing::Fresh(trail) => walk.traced(&trail).expanded(path, name, domain),
            Tracing::Repeated => Globbed::Absent,
        }
    }

    fn found(self, space: &Namespace, site: Site, domain: Domain, name: &Name) -> Globbed {
        match self {
            Tracing::Fresh(trail) => Search { space, site, domain, trail: &trail }.levels(name),
            Tracing::Repeated => Globbed::Present(Binding::Unknown),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Site {
    At(ScopeId),
    Path { module: ScopeId, viewer: ScopeId },
    Glob { module: ScopeId, viewer: ScopeId },
}

impl Site {
    fn scope(self) -> ScopeId {
        match self {
            Site::At(scope) => scope,
            Site::Path { module, .. } | Site::Glob { module, .. } => module,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Walk<'walk> {
    space: &'walk Namespace,
    scope: ScopeId,
    trail: &'walk Trail<'walk>,
}

impl<'walk> Walk<'walk> {
    pub(super) fn space(&self) -> &'walk Namespace {
        self.space
    }

    pub(super) fn scope(&self) -> ScopeId {
        self.scope
    }

    pub(super) fn path(&self, path: &UsePath) -> Binding {
        path.tail().iter().fold(path.head().started(self), |binding, segment| binding.stepped(segment, self))
    }

    pub(super) fn macro_named(&self, first: &Name) -> Binding {
        self.looked_up(Site::At(self.scope), Domain::Macros, first)
            .settled(Binding::from(StdNames::canonical(Table::Derives, first)))
    }

    pub(super) fn named(&self, first: &Name, shape: Shape) -> Binding {
        let prelude = Anchor::Crate.anchored(self.space.anchors(self.scope)).and_then(|root| {
            self.space.prelude(root, first, &Walk { space: self.space, scope: root, trail: self.trail })
        });
        self.looked_up(Site::At(self.scope), Domain::Types, first)
            .settled(prelude.unwrap_or_else(|| shape.fallback(first, self.space)))
    }

    pub(super) fn globbed_member(&self, module: ScopeId, name: &Name, domain: Domain) -> Globbed {
        self.looked_up(Site::Glob { module, viewer: self.scope }, domain, name)
    }

    pub(super) fn entered(&self, module: ScopeId, segment: &Name) -> Binding {
        match Anchor::spelled(segment) {
            Some(anchor) => Binding::from(anchor.anchored(self.space.anchors(module))),
            None => self
                .looked_up(Site::Path { module, viewer: self.scope }, Domain::Types, segment)
                .settled(Binding::Unknown),
        }
    }

    pub(super) fn expanded(&self, path: &UsePath, name: &Name, domain: Domain) -> Globbed {
        self.path(path).globbed(name, self, domain)
    }

    pub(super) fn expanding<'path>(&self, path: &'path UsePath) -> Tracing<'path>
    where
        'walk: 'path,
    {
        Expansion { scope: self.scope, path }.traced(self.trail)
    }

    fn looked_up(&self, site: Site, domain: Domain, name: &Name) -> Globbed {
        Step { scope: site.scope(), name, domain }.traced(self.trail).found(self.space, site, domain, name)
    }

    fn traced<'trail>(&self, trail: &'trail Trail<'trail>) -> Walk<'trail>
    where
        'walk: 'trail,
    {
        Walk { space: self.space, scope: self.scope, trail }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Search<'search> {
    space: &'search Namespace,
    site: Site,
    domain: Domain,
    trail: &'search Trail<'search>,
}

impl<'search> Search<'search> {
    pub(super) fn domain(&self) -> Domain {
        self.domain
    }

    pub(super) fn sight(&self, level: ScopeId) -> Sight {
        match self.site {
            Site::At(_) => Sight::Inside,
            Site::Path { viewer, .. } => Sight::ByPath.beyond(self.space.within_reach(level, viewer)),
            Site::Glob { viewer, .. } => Sight::ByGlob.beyond(self.space.within_reach(level, viewer)),
        }
    }

    pub(super) fn importer(&self, level: ScopeId) -> Walk<'search> {
        Walk { space: self.space, scope: level, trail: self.trail }
    }

    fn levels(&self, name: &Name) -> Globbed {
        self.space
            .chain(self.site.scope())
            .iter()
            .fold(Globbed::Absent, |found, level| found.or_else(|| self.space.level(*level, name, self)))
    }
}

#[derive(Debug)]
pub(crate) struct Resolver<'space> {
    space: &'space Namespace,
}

impl<'space> Resolver<'space> {
    pub(crate) fn new(space: &'space Namespace) -> Resolver<'space> {
        Resolver { space }
    }

    pub(crate) fn path_resolved(&self, scope: ScopeId, path: &UsePath, domain: Domain) -> Resolved {
        let trail = Trail::EMPTY;
        path.bound(domain, &Walk { space: self.space, scope, trail: &trail }).resolved(self.space, path)
    }
}
