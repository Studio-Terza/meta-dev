use std::path::{Path, PathBuf};

use file_ports::Files;
use lint_roots::Followed;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Child {
    path: PathBuf,
    children_directory: PathBuf,
}

impl Child {
    #[must_use]
    pub fn new(path: PathBuf, children_directory: PathBuf) -> Child {
        Child { path, children_directory }
    }

    pub fn canonically<Output>(self, files: &impl Files, then: impl FnOnce(&Path, &Path) -> Output) -> Output {
        let canonical = files.canonical(&self.path).unwrap_or(self.path);
        then(&canonical, &self.children_directory)
    }

    #[must_use]
    pub fn existing(self, files: &impl Files) -> Option<Child> {
        Some(self).filter(|child| Followed::file(files, &child.path).is_some())
    }
}
