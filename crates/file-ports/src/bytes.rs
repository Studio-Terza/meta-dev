use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bytes(Vec<u8>);

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<&str> for Bytes {
    fn from(text: &str) -> Bytes {
        Bytes(text.as_bytes().to_vec())
    }
}
