use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JobId(i64);
