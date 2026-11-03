use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct Revision(String);

impl AsRef<str> for Revision {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
