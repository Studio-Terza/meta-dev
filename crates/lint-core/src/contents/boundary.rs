use file_ports::{Bytes, FileFailure, Text};

use super::Contents;

impl From<Result<Bytes, FileFailure>> for Contents {
    fn from(read: Result<Bytes, FileFailure>) -> Self {
        match read {
            Ok(bytes) => Self::Read(bytes),
            Err(failure) => Self::Unreadable(Text::new(failure.to_string())),
        }
    }
}
