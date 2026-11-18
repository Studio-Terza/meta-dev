use std::path::Path;
use std::sync::Arc;

use algebra::Monoid;
use graph::{Aliases, Call, FileId, Found, Function, Holding, Implementation, Law, ModelPart, Region};
use lint_layout::Position;

use crate::namespace::{Resolving, ScopeKey, ScopeMap};

use super::Grown;
use super::places::Places;

pub(super) trait Lowers {
    fn lowered(self, footing: Footing<'_>) -> ModelPart;
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Footing<'grown> {
    resolving: Resolving<'grown, 'grown, 'grown>,
    file: FileId,
    outer: Region,
    places: &'grown Places,
    scopes: &'grown ScopeMap,
    aliases: &'grown Aliases,
}

impl<'grown> Footing<'grown> {
    pub(super) fn new(
        resolving: Resolving<'grown, 'grown, 'grown>,
        grown: &'grown Grown,
        places: &'grown Places,
        aliases: &'grown Aliases,
    ) -> Footing<'grown> {
        Footing { resolving, file: grown.file, outer: grown.outer, places, scopes: &grown.scopes, aliases }
    }

    pub(super) fn aliases(self) -> &'grown Aliases {
        self.aliases
    }

    pub(super) fn every<Row: Lowers>(self, rows: Vec<Row>) -> ModelPart {
        Monoid::concat(rows.into_iter().map(|row| row.lowered(self)))
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

    pub(super) fn scopes(self) -> &'grown ScopeMap {
        self.scopes
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

    pub(super) fn holding(self, footing: Footing<'_>, holding: Holding) -> ModelPart {
        ModelPart::holding(footing.file, self.position.line(), holding)
    }

    pub(super) fn law(self, footing: Footing<'_>, law: Law) -> ModelPart {
        ModelPart::law(footing.file, self.position.line(), law)
    }

    pub(super) fn call(self, footing: Footing<'_>, call: Call) -> ModelPart {
        ModelPart::call(footing.file, self.position.line(), call)
    }

    pub(super) fn implementation(self, footing: Footing<'_>, implementation: Implementation) -> ModelPart {
        ModelPart::implementation(footing.file, Found::new(self.position.line(), implementation))
    }

    pub(super) fn function(self, footing: Footing<'_>, function: Function) -> ModelPart {
        ModelPart::function(footing.file, self.position.line(), function)
    }
}
