mod boundary;

use newtype::Listed;
use serde::Deserialize;
use strum::{Display, IntoStaticStr, VariantArray};

pub use boundary::{CrateName, FeatureName};

#[derive(Display, IntoStaticStr, VariantArray, Listed, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Consideration {
    Checks,
    Process,
    Shipping,
}

#[derive(Display, IntoStaticStr, VariantArray, Listed, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Holds,
    Violated,
    Unclear,
}

impl Verdict {
    #[must_use]
    pub fn unproven(self) -> Verdict {
        match self {
            Verdict::Holds => Verdict::Holds,
            Verdict::Violated | Verdict::Unclear => Verdict::Unclear,
        }
    }
}
