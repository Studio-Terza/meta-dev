use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrateFile(PathBuf);

impl CrateFile {
    pub const UNPLACED: Self = Self(PathBuf::new());
}

impl From<PathBuf> for CrateFile {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl AsRef<Path> for CrateFile {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
