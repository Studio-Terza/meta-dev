use named_derive::Named;

use crate::Stage;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Basis {
    Set,
    Decided,
    Vague,
}

impl Basis {
    #[must_use]
    pub fn staged(self, stored: Stage) -> Stage {
        match self {
            Basis::Set => stored,
            Basis::Decided => Stage::Refining,
            Basis::Vague => Stage::Vague,
        }
    }
}
