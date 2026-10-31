use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Child {
    path: PathBuf,
    children_directory: PathBuf,
}

impl Child {
    #[must_use]
    pub fn new(path: PathBuf, children_directory: PathBuf) -> Child {
        Child { path, children_directory }
    }

    pub fn canonically<Output>(self, then: impl FnOnce(&Path, &Path) -> Output) -> Output {
        let canonical = fs::canonicalize(&self.path).unwrap_or(self.path);
        then(&canonical, &self.children_directory)
    }

    #[must_use]
    pub fn exists(&self) -> bool {
        self.path.is_file()
    }
}
