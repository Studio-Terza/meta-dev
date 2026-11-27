use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug)]
pub(crate) struct Contents(String);
