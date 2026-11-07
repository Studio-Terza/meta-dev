use newtype::{Arbitrary, Newtype};
use serde::Deserialize;
use text_lines::OneLine;

#[derive(Newtype, Arbitrary, Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackageId(OneLine);
