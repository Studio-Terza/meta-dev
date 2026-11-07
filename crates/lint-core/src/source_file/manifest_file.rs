use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestFile(PathBuf);

impl From<PathBuf> for ManifestFile {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl AsRef<Path> for ManifestFile {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
