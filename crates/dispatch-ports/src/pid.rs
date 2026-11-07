mod boundary;

use named_derive::Named;
use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pid(u64);

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ended {
    #[name("killed")]
    Killed,
    #[name("absent")]
    Absent,
}
