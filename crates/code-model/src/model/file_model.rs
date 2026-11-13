use std::path::{Path, PathBuf};

use lint_layout::Name;

#[derive(Debug)]
pub(crate) struct FileModel {
    path: PathBuf,
    krate: Name,
}

impl FileModel {
    pub(crate) fn new(path: PathBuf, krate: Name) -> FileModel {
        FileModel { path, krate }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn krate(&self) -> &Name {
        &self.krate
    }
}
