use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
pub struct Line(String);

impl AsRef<str> for Line {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
