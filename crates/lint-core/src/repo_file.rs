use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RepoFile {
    shown: PathBuf,
    inside: PathBuf,
}

impl RepoFile {
    #[must_use]
    pub fn new(shown: impl Into<PathBuf>, inside: impl Into<PathBuf>) -> RepoFile {
        RepoFile { shown: shown.into(), inside: inside.into() }
    }

    #[must_use]
    pub fn shown(&self) -> &Path {
        &self.shown
    }

    #[must_use]
    pub fn lies_under(&self, directory: &Path) -> bool {
        self.inside
            .parent()
            .is_some_and(|parents| parents.components().any(|part| part.as_os_str() == directory.as_os_str()))
    }

    #[must_use]
    pub fn below(&self, directory: &Path) -> Option<&Path> {
        self.inside.strip_prefix(directory).ok()
    }
}
