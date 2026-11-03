mod boundary;

use file_ports::{Bytes, Text};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Contents {
    Read(Bytes),
    Unreadable(Text),
}

impl Contents {
    pub fn read(&self) -> Result<&Bytes, &Text> {
        match self {
            Contents::Read(bytes) => Ok(bytes),
            Contents::Unreadable(reason) => Err(reason),
        }
    }
}
