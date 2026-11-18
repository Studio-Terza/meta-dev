use std::path::{Path, PathBuf};

use crate::Bytes;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Written {
    path: PathBuf,
    bytes: Bytes,
}

impl Written {
    #[must_use]
    pub fn new(path: PathBuf, bytes: Bytes) -> Written {
        Written { path, bytes }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn into_bytes(self) -> Bytes {
        self.bytes
    }
}
