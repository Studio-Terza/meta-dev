use std::path::{Path, PathBuf};

use crate::SessionId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Session {
    stem: SessionId,
    file: PathBuf,
}

impl Session {
    pub(crate) fn found(stem: SessionId, file: PathBuf) -> Session {
        Session { stem, file }
    }

    #[must_use]
    pub fn stem(&self) -> &SessionId {
        &self.stem
    }

    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }
}
