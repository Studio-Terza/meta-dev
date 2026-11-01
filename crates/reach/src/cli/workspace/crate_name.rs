use newtype::{Display, Newtype};
use serde::Deserialize;

#[derive(Newtype, Display, Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrateName(String);
