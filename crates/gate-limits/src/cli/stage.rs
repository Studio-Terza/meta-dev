use newtype::{Display, Newtype};
use serde::Deserialize;
use text_lines::OneLine;

#[derive(Newtype, Display, Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Stage(OneLine);

impl AsRef<str> for Stage {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
