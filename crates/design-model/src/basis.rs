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
    pub fn implied(self) -> Option<Stage> {
        match self {
            Basis::Set => None,
            Basis::Decided => Some(Stage::Refining),
            Basis::Vague => Some(Stage::Vague),
        }
    }
}
