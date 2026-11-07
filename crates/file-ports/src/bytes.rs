use std::fmt;
use std::io::Write;

use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bytes(Vec<u8>);

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<Shown: fmt::Display + ?Sized> From<&Shown> for Bytes {
    fn from(shown: &Shown) -> Self {
        let mut bytes = Vec::new();
        write!(bytes, "{shown}").map_or_else(|_| Self::default(), |()| Self(bytes))
    }
}
