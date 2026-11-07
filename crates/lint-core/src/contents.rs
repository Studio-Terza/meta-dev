mod boundary;

use file_ports::{Bytes, Text};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Contents {
    Read(Bytes),
    Unreadable(Text),
}

impl Contents {
    pub const fn read(&self) -> Result<&Bytes, &Text> {
        match self {
            Self::Read(bytes) => Ok(bytes),
            Self::Unreadable(reason) => Err(reason),
        }
    }
}
