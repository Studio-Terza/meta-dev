use std::io;
use std::path::{Path, PathBuf};

use crate::failure::Failure;
use crate::name::Name;
use crate::prefix::Prefix;

#[derive(Debug)]
pub struct Directory(PathBuf);

impl Directory {
    const BUILD_DIRECTORY: &'static str = "target";
    const MANIFEST: &'static str = "Cargo.toml";
    const HIDDEN: &'static str = ".";

    #[must_use]
    pub fn new(path: PathBuf) -> Directory {
        Directory(path)
    }

    #[must_use]
    pub fn is_real(path: &Path) -> bool {
        std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_dir())
    }

    pub fn entries(&self) -> Result<Vec<PathBuf>, Failure> {
        let failed = |source: io::Error| Failure::Io { path: self.0.clone(), source };
        std::fs::read_dir(&self.0)
            .map_err(failed)?
            .map(|entry| entry.map(|found| found.path()).map_err(failed))
            .collect::<Result<Vec<PathBuf>, Failure>>()
            .map(|mut entries| {
                entries.sort();
                entries
            })
    }

    pub fn tree(self) -> Result<Vec<Directory>, Failure> {
        let below = self.entries()?;
        below
            .into_iter()
            .filter(|entry| Self::is_real(entry))
            .map(Directory)
            .filter(|directory| !directory.is_skipped())
            .try_fold(vec![self], |found, directory| Ok(found.into_iter().chain(directory.tree()?).collect()))
    }

    fn is_skipped(&self) -> bool {
        self.0.file_name().is_some_and(|name| {
            Name::from(name).matches::<Prefix>(&[Self::HIDDEN])
                || (name == Self::BUILD_DIRECTORY
                    && self.0.parent().is_some_and(|parent| parent.join(Self::MANIFEST).is_file()))
        })
    }
}

impl AsRef<Path> for Directory {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
