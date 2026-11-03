use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct PullNumber(String);

impl AsRef<str> for PullNumber {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
