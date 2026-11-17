use std::path::Path;
use std::sync::Arc;

use graph::{Building, Call, FileId, Found, Function, Holding, Implementation, Law, Region};
use lint_layout::Position;

use crate::namespace::{Resolving, ScopeId, ScopeKey, ScopeMap};

use super::places::Places;

pub(super) trait Built {
    fn built(self, footing: Footing<'_>, building: Building) -> Building;
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Footing<'grown> {
    resolving: Resolving<'grown, 'grown, 'grown>,
    file: FileId,
    outer: Region,
    places: &'grown Places,
    scopes: &'grown ScopeMap,
}

impl<'grown> Footing<'grown> {
    pub(super) fn new(
        resolving: Resolving<'grown, 'grown, 'grown>,
        file: FileId,
        outer: Region,
        places: &'grown Places,
        scopes: &'grown ScopeMap,
    ) -> Footing<'grown> {
        Footing { resolving, file, outer, places, scopes }
    }

    pub(super) fn resolving(self) -> Resolving<'grown, 'grown, 'grown> {
        self.resolving
    }

    pub(super) fn places(self) -> &'grown Places {
        self.places
    }

    pub(super) fn outer(self) -> Region {
        self.outer
    }

    pub(super) fn scope(self, key: &ScopeKey) -> ScopeId {
        self.scopes.scope(key)
    }

    pub(super) fn file(self) -> FileId {
        self.file
    }
}

#[derive(Clone, Debug)]
pub(super) struct Seat {
    scope: ScopeKey,
    children: Arc<Path>,
}

impl Seat {
    pub(super) fn new(scope: ScopeKey, children: Arc<Path>) -> Seat {
        Seat { scope, children }
    }

    pub(super) fn scope(&self) -> &ScopeKey {
        &self.scope
    }

    pub(super) fn children(&self) -> Arc<Path> {
        Arc::clone(&self.children)
    }

    pub(super) fn located(&self, position: Position) -> Located {
        Located { position, scope: self.scope.clone() }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Located {
    position: Position,
    scope: ScopeKey,
}

impl Located {
    pub(super) fn scope(&self) -> &ScopeKey {
        &self.scope
    }

    pub(super) fn position(&self) -> Position {
        self.position
    }

    pub(super) fn holding(self, footing: Footing<'_>, mut building: Building, holding: Holding) -> Building {
        building.holding(footing.file, self.position.line(), holding);
        building
    }

    pub(super) fn law(self, footing: Footing<'_>, mut building: Building, law: Law) -> Building {
        building.law(footing.file, self.position.line(), law);
        building
    }

    pub(super) fn call(self, footing: Footing<'_>, mut building: Building, call: Call) -> Building {
        building.call(footing.file, self.position.line(), call);
        building
    }

    pub(super) fn implementation(
        self,
        footing: Footing<'_>,
        mut building: Building,
        implementation: Implementation,
    ) -> Building {
        building.implementation(footing.file, Found::new(self.position.line(), implementation));
        building
    }

    pub(super) fn function(self, footing: Footing<'_>, mut building: Building, function: Function) -> Building {
        building.function(footing.file, self.position.line(), function);
        building
    }
}
