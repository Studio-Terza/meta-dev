use std::path::{Path, PathBuf};

use crate::contents::Contents;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RepoFile {
    shown: PathBuf,
    inside: PathBuf,
    contents: Contents,
}

impl RepoFile {
    #[must_use]
    pub fn new(shown: impl Into<PathBuf>, inside: impl Into<PathBuf>, contents: Contents) -> Self {
        Self { shown: shown.into(), inside: inside.into(), contents }
    }

    #[must_use]
    pub fn contents(&self) -> &Contents {
        &self.contents
    }

    #[must_use]
    pub fn shown(&self) -> &Path {
        &self.shown
    }

    #[must_use]
    pub fn lies_under(&self, directory: &Path) -> Option<&Self> {
        self.inside
            .parent()
            .filter(|parents| parents.components().any(|part| part.as_os_str() == directory.as_os_str()))
            .map(|_| self)
    }

    #[must_use]
    pub fn beneath_named(&self, directory: &Path) -> Option<PathBuf> {
        let mut parts = self.inside.components();
        parts.by_ref().find(|part| part.as_os_str() == directory.as_os_str())?;
        Some(parts.as_path().to_path_buf())
    }

    #[must_use]
    pub fn below(&self, directory: &Path) -> Option<&Path> {
        self.inside.strip_prefix(directory).ok()
    }
}
