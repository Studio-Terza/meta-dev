use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::directory::Directory;
use super::target::Target;
use crate::failure::Failure;
use crate::source_files::tree::Tree;

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

    pub fn targets_under(&self, tree: &Tree) -> Result<Vec<Target>, Failure> {
        let roots: Vec<PathBuf> =
            Self::ROOT_FILES.iter().map(|file| self.directory.join(file)).filter(|file| file.is_file()).collect();
        let roots: Vec<PathBuf> = self.declared.iter().filter(|file| file.is_file()).cloned().chain(roots).collect();
        let files = Self::TARGET_DIRECTORIES.iter().try_fold(roots, |found, directory| {
            Ok(found.into_iter().chain(Self::crate_roots(&self.directory.join(directory))?).collect::<Vec<PathBuf>>())
        })?;
        Ok(files
            .into_iter()
            .filter(|file| file.starts_with(tree.canonical()))
            .map(|file| Target::new(self.directory.clone(), file))
            .collect())
    }

    fn crate_roots(directory: &Path) -> Result<Vec<PathBuf>, Failure> {
        Some(directory).filter(|candidate| Directory::is_real(candidate)).map_or_else(
            || Ok(Vec::new()),
            |real| Ok(Directory::new(real.to_path_buf()).entries()?.into_iter().filter_map(Self::crate_root).collect()),
        )
    }

    fn crate_root(entry: PathBuf) -> Option<PathBuf> {
        if Directory::is_real(&entry) {
            Some(entry.join(Self::BINARY_ROOT)).filter(|main| main.is_file())
        } else {
            Some(entry).filter(|file| file.extension() == Some(OsStr::new(Self::SOURCE_EXTENSION)))
        }
    }
}
