use newtype::{Arbitrary, Display, Newtype};
use one_line::OneLine;
use serde::Deserialize;

#[derive(Newtype, Arbitrary, Display, Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrateName(OneLine);

impl AsRef<str> for CrateName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
