use newtype::{Arbitrary, Display, Newtype};
use serde::Serialize;

#[derive(Newtype, Arbitrary, Display, Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct TestName(String);
