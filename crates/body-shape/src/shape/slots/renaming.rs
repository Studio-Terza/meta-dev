use equivalence::{Equivalence, Erased};
use lint_layout::{Casing, Name};

use crate::shape::position::Position;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Renaming {
    Slotted,
    Kept,
}

impl Renaming {
    pub(super) fn chosen(name: &Name, position: Position, equivalence: Equivalence) -> Self {
        equivalence
            .erases(Erased::Names)
            .map(|_| Self::Slotted)
            .or_else(|| equivalence.erases(Erased::Locals).and_then(|_| Self::local(name.casing(), position)))
            .unwrap_or(Self::Kept)
    }

    fn local(casing: Casing, position: Position) -> Option<Self> {
        Some(casing).filter(|casing| *casing == Casing::Lower).and(position.plain()).map(|_| Self::Slotted)
    }
}
