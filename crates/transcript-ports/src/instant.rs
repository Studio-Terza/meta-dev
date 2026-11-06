use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(String);
