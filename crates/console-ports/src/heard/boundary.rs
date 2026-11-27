use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, serde::Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(transparent)]
pub struct Heard(String);
