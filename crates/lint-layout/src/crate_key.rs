use newtype::{Arbitrary, Newtype};

use crate::name::Name;

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CrateKey(Vec<Name>);
