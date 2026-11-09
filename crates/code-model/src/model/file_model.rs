use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct FileModel {
    path: PathBuf,
}

impl From<PathBuf> for FileModel {
    fn from(path: PathBuf) -> FileModel {
        FileModel { path }
    }
}

impl FileModel {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}
