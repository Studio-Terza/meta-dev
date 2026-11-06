use code_model::Resolved;
use lint_layout::Name;

use super::Namespace;
use super::anchor::Anchor;
use super::binding::{Binding, Globbed};
use super::boundary::ScopeId;
use super::entries::Visibility;
use super::use_path::{Shape, UsePath};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Step {
    scope: ScopeId,
    name: Name,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Expansion {
    scope: ScopeId,
    path: UsePath,
}

#[derive(Clone, Debug, Default)]
pub(super) struct Trail {
    visited: Vec<Step>,
    expanding: Vec<Expansion>,
}

impl Step {
    const LIMIT: usize = 64;

    fn traced(&self, trail: &Trail) -> Tracing {
        match trail.visited.iter().find(|visited| *visited == self).or_else(|| trail.visited.get(Self::LIMIT)) {
            Some(_) => Tracing::Repeated,
            None => Tracing::Fresh(Trail {
                visited: trail.visited.iter().cloned().chain([self.clone()]).collect(),
                expanding: trail.expanding.clone(),
            }),
        }
    }
}

impl Expansion {
    fn traced(&self, trail: &Trail) -> Tracing {
        match trail.expanding.iter().find(|expanding| *expanding == self) {
            Some(_) => Tracing::Repeated,
            None => Tracing::Fresh(Trail {
                visited: Vec::new(),
                expanding: trail.expanding.iter().cloned().chain([self.clone()]).collect(),
            }),
        }
    }
}

#[derive(Debug)]
pub(super) enum Tracing {
    Fresh(Trail),
    Repeated,
}

impl Tracing {
    pub(super) fn globbed(self, walk: &Walk<'_>, path: &UsePath, name: &Name) -> Globbed {
        match self {
            Tracing::Fresh(trail) => walk.traced(trail).expanded(path, name),
            Tracing::Repeated => Globbed::Absent,
        }
    }

    fn found(self, walk: &Walk<'_>, name: &Name) -> Globbed {
        match self {
            Tracing::Fresh(trail) => walk.traced(trail).levels(name),
            Tracing::Repeated => Globbed::Present(Binding::Unknown),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Walk<'space> {
    space: &'space Namespace,
    scope: ScopeId,
    from: ScopeId,
    trail: Trail,
}

impl<'space> Walk<'space> {
    pub(super) fn space(&self) -> &'space Namespace {
        self.space
    }

    pub(super) fn scope(&self) -> ScopeId {
        self.scope
    }

    pub(super) fn path(&self, path: &UsePath) -> Binding {
        path.tail().iter().fold(path.head().started(self), |binding, segment| binding.stepped(segment, self))
    }

    pub(super) fn named(&self, first: &Name, shape: Shape) -> Binding {
        self.visiting(first).found(self, first).settled(shape.fallback(first, self.space))
    }

    pub(super) fn member(&self, module: ScopeId, name: &Name) -> Globbed {
        let walk = Walk { scope: module, from: self.scope, ..self.clone() };
        walk.visiting(name).found(&walk, name)
    }

    pub(super) fn entered(&self, module: ScopeId, segment: &Name) -> Binding {
        match Anchor::spelled(segment) {
            Some(anchor) => Binding::from(anchor.anchored(self.space.anchors(module))),
            None => self.member(module, segment).settled(Binding::Unknown),
        }
    }

    pub(super) fn expanded(&self, path: &UsePath, name: &Name) -> Globbed {
        self.path(path).globbed(name, self)
    }

    pub(super) fn seen(&self, level: ScopeId) -> &'static [Visibility] {
        self.space.reach(level, self.from).seen()
    }

    pub(super) fn at_level(&self, scope: ScopeId) -> Walk<'space> {
        Walk { scope, from: scope, ..self.clone() }
    }

    pub(super) fn expanding(&self, path: &UsePath) -> Tracing {
        Expansion { scope: self.scope, path: path.clone() }.traced(&self.trail)
    }

    fn visiting(&self, name: &Name) -> Tracing {
        Step { scope: self.scope, name: name.clone() }.traced(&self.trail)
    }

    fn traced(&self, trail: Trail) -> Walk<'space> {
        Walk { trail, ..self.clone() }
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

    pub(crate) fn path_resolved(&self, scope: ScopeId, path: &UsePath) -> Resolved {
        let walk = Walk { space: self.space, scope, from: scope, trail: Trail::default() };
        walk.path(path).resolved(self.space, path)
    }
}
