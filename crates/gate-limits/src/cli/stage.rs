use newtype::{Display, Newtype};
use serde::Deserialize;
use text_lines::OneLine;

#[derive(Newtype, Display, Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Stage(OneLine);
