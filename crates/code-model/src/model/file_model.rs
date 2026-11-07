use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct FileModel {
    path: PathBuf,
}

impl From<PathBuf> for FileModel {
    fn from(path: PathBuf) -> Self {
        Self { path }
    }
}

impl FileModel {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}
