use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct Branch(String);

impl AsRef<str> for Branch {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
