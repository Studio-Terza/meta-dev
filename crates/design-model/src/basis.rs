use newtype::Listed;
use strum::{Display, IntoStaticStr, VariantArray};

use crate::Stage;

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
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
