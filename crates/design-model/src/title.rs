mod boundary;

use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct Title(String);
