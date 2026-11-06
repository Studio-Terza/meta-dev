use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scratch(PathBuf);

impl From<PathBuf> for Scratch {
    fn from(directory: PathBuf) -> Scratch {
        Scratch(directory)
    }
}

impl AsRef<Path> for Scratch {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
