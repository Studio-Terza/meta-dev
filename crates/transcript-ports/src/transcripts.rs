use file_ports::FileFailure;
use port_core::port;

use crate::{Cursor, Read, ReadFailure, Session};

#[port]
pub trait Transcripts {
    fn sessions(&self) -> Result<Vec<Session>, FileFailure>;

    fn read_from(&self, file: &Session, cursor: Cursor) -> Result<Read, ReadFailure>;
}
