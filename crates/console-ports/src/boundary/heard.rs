use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug, Default, PartialEq, Eq)]
pub struct Heard(String);
