use newtype::{Arbitrary, Display, Newtype};
use one_line::OneLine;
use serde::Deserialize;

#[derive(Newtype, Arbitrary, Display, Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Stage(OneLine);

impl AsRef<str> for Stage {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
