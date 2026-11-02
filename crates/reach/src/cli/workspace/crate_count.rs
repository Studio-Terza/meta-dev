use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrateCount(usize);
