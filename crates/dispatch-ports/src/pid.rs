mod boundary;

use newtype::{Arbitrary, Display, Listed, Newtype};
use strum::{IntoStaticStr, VariantArray};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pid(u32);

#[derive(strum::Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Ended {
    Killed,
    Absent,
}
