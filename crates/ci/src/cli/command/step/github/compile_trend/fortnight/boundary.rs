use newtype::{Arbitrary, Display, Newtype};
use serde::Deserialize;

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Deserialize)]
#[serde(transparent)]
pub(super) struct RunId(u64);
