use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mutant(String);
