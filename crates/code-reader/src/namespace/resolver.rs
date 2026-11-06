use code_model::Resolved;
use lint_layout::Name;

use super::Namespace;
use super::anchor::Anchor;
use super::binding::{Binding, Globbed};
use super::boundary::ScopeId;
use super::entries::{Sight, Via};
use super::std_names::{StdNames, Table};
use super::use_path::{Shape, UsePath};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Domain {
    Types,
    Macros,
}

impl Domain {
    pub(super) fn typed(self, found: impl FnOnce() -> Globbed) -> Globbed {
        match self {
            Domain::Types => found(),
            Domain::Macros => Globbed::Absent,
        }
    }

    fn bound(self, walk: &Walk<'_>, path: &UsePath) -> Binding {
        match self {
            Domain::Types => walk.path(path),
            Domain::Macros => path.head().derived(walk, path),
        }
    }
}

impl Via {
    fn domain(self, current: Domain) -> Domain {
        match self {
            Via::Path => Domain::Types,
            Via::Glob => current,
        }
    }
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
    pub(super) fn globbed(self, walk: &Walk<'_>, path: &UsePath, name: &Name) -> Globbed {
        match self {
            Tracing::Fresh(trail) => walk.traced(&trail).expanded(path, name),
            Tracing::Repeated => Globbed::Absent,
        }
    }

    fn found(self, walk: &Walk<'_>, name: &Name) -> Globbed {
        match self {
            Tracing::Fresh(trail) => walk.traced(&trail).levels(name),
            Tracing::Repeated => Globbed::Present(Binding::Unknown),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Walk<'walk> {
    space: &'walk Namespace,
    scope: ScopeId,
    from: ScopeId,
    via: Via,
    domain: Domain,
    trail: &'walk Trail<'walk>,
}

impl<'walk> Walk<'walk> {
    pub(super) fn space(&self) -> &'walk Namespace {
        self.space
    }

    pub(super) fn scope(&self) -> ScopeId {
        self.scope
    }

    pub(super) fn domain(&self) -> Domain {
        self.domain
    }

    pub(super) fn path(&self, path: &UsePath) -> Binding {
        let walk = Walk { domain: Domain::Types, ..*self };
        path.tail().iter().fold(path.head().started(&walk), |binding, segment| binding.stepped(segment, &walk))
    }

    pub(super) fn macro_named(&self, first: &Name) -> Binding {
        let walk = Walk { domain: Domain::Macros, ..*self };
        walk.visiting(first).found(&walk, first).settled(Binding::from(StdNames::canonical(Table::Derives, first)))
    }

    pub(super) fn named(&self, first: &Name, shape: Shape) -> Binding {
        let prelude = Anchor::Crate
            .anchored(self.space.anchors(self.scope))
            .and_then(|root| self.space.prelude(root, first, &self.at_level(root)));
        self.visiting(first).found(self, first).settled(prelude.unwrap_or_else(|| shape.fallback(first, self.space)))
    }

    pub(super) fn member(&self, module: ScopeId, name: &Name, via: Via) -> Globbed {
        let walk = Walk { scope: module, from: self.scope, via, domain: via.domain(self.domain), ..*self };
        walk.visiting(name).found(&walk, name)
    }

    pub(super) fn entered(&self, module: ScopeId, segment: &Name) -> Binding {
        match Anchor::spelled(segment) {
            Some(anchor) => Binding::from(anchor.anchored(self.space.anchors(module))),
            None => self.member(module, segment, Via::Path).settled(Binding::Unknown),
        }
    }

    pub(super) fn expanded(&self, path: &UsePath, name: &Name) -> Globbed {
        self.path(path).globbed(name, self)
    }

    pub(super) fn view(&self, level: ScopeId) -> Sight {
        match self.space.within_reach(level, self.from) {
            Some(_) => Sight::Inside,
            None => self.via.outside(),
        }
    }

    pub(super) fn at_level(&self, scope: ScopeId) -> Walk<'walk> {
        Walk { scope, from: scope, ..*self }
    }

    pub(super) fn expanding<'path>(&self, path: &'path UsePath) -> Tracing<'path>
    where
        'walk: 'path,
    {
        Expansion { scope: self.scope, path }.traced(self.trail)
    }

    fn visiting<'name>(&self, name: &'name Name) -> Tracing<'name>
    where
        'walk: 'name,
    {
        Step { scope: self.scope, name, domain: self.domain }.traced(self.trail)
    }

    fn traced<'trail>(&self, trail: &'trail Trail<'trail>) -> Walk<'trail>
    where
        'walk: 'trail,
    {
        Walk { trail, ..*self }
    }

    fn levels(&self, name: &Name) -> Globbed {
        self.space
            .chain(self.scope)
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
        let walk = Walk { space: self.space, scope, from: scope, via: Via::Path, domain: Domain::Types, trail: &trail };
        domain.bound(&walk, path).resolved(self.space, path)
    }
}
