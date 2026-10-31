use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrateRoot(PathBuf);

impl CrateRoot {
    #[must_use]
    pub fn new(directory: PathBuf) -> CrateRoot {
        CrateRoot(directory)
    }

    #[must_use]
    pub fn directory_name(&self) -> Option<&OsStr> {
        self.0.file_name()
    }
}
