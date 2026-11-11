use std::path::{Path, PathBuf};

use file_ports::Size;
use newtype::Wrapping;

use crate::{Cursor, SessionId};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Session {
    stem: SessionId,
    file: PathBuf,
    size: Size,
}

impl Session {
    pub(crate) fn found(stem: SessionId, file: PathBuf, size: Size) -> Session {
        Session { stem, file, size }
    }

    #[must_use]
    pub fn stem(&self) -> &SessionId {
        &self.stem
    }

    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }

    #[must_use]
    pub fn unread(&self, cursor: Cursor) -> Option<Cursor> {
        (self.size.into_inner() > cursor.into_inner().into_inner()).then_some(cursor)
    }
}
