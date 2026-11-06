use std::io;

use newtype::{Arbitrary, Display, Newtype};

use crate::{Bytes, FileFailure};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Text(String);

impl AsRef<str> for Text {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&Bytes> for Text {
    type Error = FileFailure;

    fn try_from(bytes: &Bytes) -> Result<Text, FileFailure> {
        String::from_utf8(bytes.as_ref().to_vec())
            .map(Text::new)
            .map_err(|_| FileFailure::Failed(io::Error::from(io::ErrorKind::InvalidData)))
    }
}
