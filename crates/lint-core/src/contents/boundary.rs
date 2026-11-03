use file_ports::{Bytes, FileFailure, Text};

use super::Contents;

impl From<Result<Bytes, FileFailure>> for Contents {
    fn from(read: Result<Bytes, FileFailure>) -> Contents {
        match read {
            Ok(bytes) => Contents::Read(bytes),
            Err(failure) => Contents::Unreadable(Text::new(failure.to_string())),
        }
    }
}
