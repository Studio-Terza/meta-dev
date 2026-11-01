use newtype::{Display, Newtype};
use serde::Deserialize;

#[derive(Newtype, Display, Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct Number(u64);
