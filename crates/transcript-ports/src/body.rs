use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Body(String);

impl AsRef<str> for Body {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
