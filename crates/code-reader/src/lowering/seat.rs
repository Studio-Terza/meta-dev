use std::path::Path;
use std::rc::Rc;

use code_model::{Building, Call, FileId, Found, Function, Holding, Implementation};
use lint_layout::Position;

use crate::namespace::{Resolving, ScopeId};

pub(super) trait Built {
    fn built(self, resolving: &Resolving<'_, '_, '_>, building: Building) -> Building;
}

#[derive(Clone, Debug)]
pub(super) struct Seat {
    file: FileId,
    scope: ScopeId,
    children: Rc<Path>,
}

impl Seat {
    pub(super) fn new(file: FileId, scope: ScopeId, children: Rc<Path>) -> Self {
        Self { file, scope, children }
    }

    pub(super) fn inner(&self, scope: ScopeId, children: Rc<Path>) -> Self {
        Self { file: self.file, scope, children }
    }

    pub(super) fn scope(&self) -> ScopeId {
        self.scope
    }

    pub(super) fn children(&self) -> Rc<Path> {
        Rc::clone(&self.children)
    }

    pub(super) fn located(&self, position: Position) -> Located {
        Located { file: self.file, position, scope: self.scope }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Located {
    file: FileId,
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

    pub(super) fn holding(self, mut building: Building, holding: Holding) -> Building {
        building.holding(self.file, self.position.line(), holding);
        building
    }

    pub(super) fn call(self, mut building: Building, call: Call) -> Building {
        building.call(self.file, self.position.line(), call);
        building
    }

    pub(super) fn implementation(self, mut building: Building, implementation: Implementation) -> Building {
        building.implementation(self.file, Found::new(self.position.line(), implementation));
        building
    }

    pub(super) fn function(self, mut building: Building, function: Function) -> Building {
        building.function(self.file, self.position.line(), function);
        building
    }
}
