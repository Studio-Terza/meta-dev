use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::directory::Directory;
use super::target::Target;
use crate::followed::Followed;
use crate::tree::Tree;
use file_ports::Files;
use lint_layout::Failure;

#[derive(Debug)]
pub struct Package {
    directory: PathBuf,
    declared: BTreeSet<PathBuf>,
}

impl Package {
    const ROOT_FILES: [&'static str; 3] = ["build.rs", "src/lib.rs", "src/main.rs"];
    const TARGET_DIRECTORIES: [&'static str; 4] = ["src/bin", "tests", "examples", "benches"];
    const BINARY_ROOT: &'static str = "main.rs";
    const SOURCE_EXTENSION: &'static str = "rs";

    #[must_use]
    pub fn in_directory(directory: PathBuf, declared: BTreeSet<PathBuf>) -> Package {
        Package { directory, declared }
    }

    pub fn targets_under(&self, files: &impl Files, tree: &Tree) -> Result<Vec<Target>, Failure> {
        let roots: Vec<PathBuf> = Self::ROOT_FILES
            .iter()
            .map(|file| self.directory.join(file))
            .filter(|file| Followed::file(files, file).is_some())
            .collect();
        let roots: Vec<PathBuf> =
            self.declared.iter().filter(|file| Followed::file(files, file).is_some()).cloned().chain(roots).collect();
        let found = Self::TARGET_DIRECTORIES.iter().try_fold(roots, |found, directory| {
            Ok(found
                .into_iter()
                .chain(Self::crate_roots(files, &self.directory.join(directory))?)
                .collect::<Vec<PathBuf>>())
        })?;
        Ok(found
            .into_iter()
            .filter(|file| file.starts_with(tree.canonical()))
            .map(|file| Target::new(self.directory.clone(), file))
            .collect())
    }

    fn crate_roots(files: &impl Files, directory: &Path) -> Result<Vec<PathBuf>, Failure> {
        Directory::real(files, directory).map_or_else(
            || Ok(Vec::new()),
            |real| {
                Ok(Directory::new(real.to_path_buf())
                    .entries(files)?
                    .into_iter()
                    .filter_map(|entry| Self::crate_root(files, entry))
                    .collect())
            },
        )
    }

    fn crate_root(files: &impl Files, entry: PathBuf) -> Option<PathBuf> {
        match Directory::real(files, &entry) {
            Some(real) => Some(real.join(Self::BINARY_ROOT)).filter(|main| Followed::file(files, main).is_some()),
            None => Some(entry).filter(|file| file.extension() == Some(OsStr::new(Self::SOURCE_EXTENSION))),
        }
    }
}
