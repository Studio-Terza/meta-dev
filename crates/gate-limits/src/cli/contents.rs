use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug)]
pub struct Contents(String);
