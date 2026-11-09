use named_derive::Named;
use strum::VariantArray;

use crate::Stage;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Basis {
    #[name("set")]
    Set,
    #[name("decided")]
    Decided,
    #[name("vague")]
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
