use std::path::{Path, PathBuf};

use crate::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Session {
    id: Uuid,
    file: PathBuf,
}

impl Session {
    pub(crate) fn found(id: Uuid, file: PathBuf) -> Session {
        Session { id, file }
    }

    #[must_use]
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }
}
