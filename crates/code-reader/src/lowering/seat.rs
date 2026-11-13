use std::path::Path;
use std::sync::Arc;

use code_model::{Building, Call, FileId, Found, Function, Holding, Implementation, Law, Region};
use lint_layout::Position;

use crate::namespace::{Resolving, ScopeId};

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
}

impl<'grown> Footing<'grown> {
    pub(super) fn new(
        resolving: Resolving<'grown, 'grown, 'grown>,
        file: FileId,
        outer: Region,
        places: &'grown Places,
    ) -> Footing<'grown> {
        Footing { resolving, file, outer, places }
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

    pub(super) fn file(self) -> FileId {
        self.file
    }
}

#[derive(Clone, Debug)]
pub(super) struct Seat {
    scope: ScopeId,
    children: Arc<Path>,
}

impl Seat {
    pub(super) fn new(scope: ScopeId, children: Arc<Path>) -> Seat {
        Seat { scope, children }
    }

    pub(super) fn scope(&self) -> ScopeId {
        self.scope
    }

    pub(super) fn children(&self) -> Arc<Path> {
        Arc::clone(&self.children)
    }

    pub(super) fn located(&self, position: Position) -> Located {
        Located { position, scope: self.scope }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Located {
    position: Position,
    scope: ScopeId,
}

impl Located {
    pub(super) fn scope(self) -> ScopeId {
        self.scope
    }

    pub(super) fn position(self) -> Position {
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
