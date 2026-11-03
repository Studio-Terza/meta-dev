use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Text(String);

impl AsRef<str> for Text {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
