use file_ports::FileFailure;
use port_core::port;

use crate::{Cursor, Read, ReadFailure, Session};

#[port]
pub trait Transcripts {
    fn sessions(&self) -> Result<Vec<Session>, FileFailure>;

    fn read_from(&self, file: &Session, cursor: Cursor) -> Result<Read, ReadFailure>;

    fn read_all(&self, files: &[Session]) -> Result<Vec<Read>, ReadFailure> {
        files.iter().map(|file| self.read_from(file, Cursor::default())).collect()
    }
}
