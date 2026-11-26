use newtype::Listed;
use strum::{Display, IntoStaticStr, VariantArray};

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Halt {
    Live,
    Stalled,
    Overrun,
}

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Fate {
    Blocked,
    Passed,
}
