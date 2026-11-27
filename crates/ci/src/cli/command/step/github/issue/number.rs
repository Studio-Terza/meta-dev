use newtype::{Arbitrary, Display, Newtype};
use serde::Deserialize;

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub(crate) struct Number(u64);
