use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uuid(String);

impl AsRef<str> for Uuid {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
