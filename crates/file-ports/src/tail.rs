use std::path::{Path, PathBuf};

use crate::Offset;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tail {
    path: PathBuf,
    offset: Offset,
}

impl Tail {
    #[must_use]
    pub fn new(path: PathBuf, offset: Offset) -> Tail {
        Tail { path, offset }
    }

    #[must_use]
    pub fn offset(&self) -> Offset {
        self.offset
    }
}

impl AsRef<Path> for Tail {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
