use newtype::{Arbitrary, Display, Newtype};
use serde::Deserialize;
use text_lines::OneLine;

#[derive(Newtype, Arbitrary, Display, Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrateName(OneLine);
