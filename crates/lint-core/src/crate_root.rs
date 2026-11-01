use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::manifest::Manifest;
use crate::name::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrateRoot {
    directory: PathBuf,
    package: Option<Name>,
}

impl CrateRoot {
    #[must_use]
    pub fn new(directory: PathBuf) -> CrateRoot {
        CrateRoot { directory, package: None }
    }

    #[must_use]
    pub fn of_package(directory: &Path) -> CrateRoot {
        let package = Manifest::read(directory).ok().flatten().and_then(|manifest| manifest.package_name());
        CrateRoot { directory: directory.to_path_buf(), package }
    }

    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    #[must_use]
    pub fn directory_name(&self) -> Option<&OsStr> {
        self.directory.file_name()
    }

    #[must_use]
    pub fn package(&self) -> Option<&Name> {
        self.package.as_ref()
    }
}
