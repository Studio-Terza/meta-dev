mod boundary;

use newtype::Listed;
use strum::{IntoStaticStr, VariantArray};

pub use boundary::Pid;

#[derive(strum::Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Ended {
    Killed,
    Absent,
}
