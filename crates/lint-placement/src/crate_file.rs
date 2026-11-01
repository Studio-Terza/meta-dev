use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrateFile(PathBuf);

impl From<PathBuf> for CrateFile {
    fn from(path: PathBuf) -> CrateFile {
        CrateFile(path)
    }
}

impl AsRef<Path> for CrateFile {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
