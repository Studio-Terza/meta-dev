use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::roots::target::Target;
use crate::survey::Survey;
use crate::tree::Tree;
use file_ports::Kind;

#[derive(Debug)]
pub struct Package {
    directory: PathBuf,
    declared: BTreeSet<PathBuf>,
}

impl Package {
    const ROOT_FILES: [&'static str; 3] = ["build.rs", "src/lib.rs", "src/main.rs"];
    const TARGET_DIRECTORIES: [&'static str; 4] = ["src/bin", "tests", "examples", "benches"];
    pub const BINARY_ROOT: &'static str = "main.rs";
    const SOURCE_EXTENSION: &'static str = "rs";

    #[must_use]
    pub fn in_directory(directory: PathBuf, declared: BTreeSet<PathBuf>) -> Package {
        Package { directory, declared }
    }

    #[must_use]
    pub fn candidates(&self) -> BTreeSet<PathBuf> {
        self.declared.iter().cloned().chain(Self::ROOT_FILES.iter().map(|file| self.directory.join(file))).collect()
    }

    #[must_use]
    pub fn target_directories(&self) -> Vec<PathBuf> {
        Self::TARGET_DIRECTORIES.iter().map(|directory| self.directory.join(directory)).collect()
    }

    #[must_use]
    pub fn into_target(self, survey: &Survey, tree: &Tree<'_>) -> Option<Target> {
        let crates: Vec<PathBuf> = self
            .target_directories()
            .iter()
            .flat_map(|directory| survey.entries(directory))
            .filter_map(|entry| Self::crate_root(survey, entry))
            .collect();
        let Package { directory, declared } = self;
        let declared = declared.into_iter().filter(|file| survey.file(file).is_some());
        let roots = Self::ROOT_FILES.iter().map(|file| directory.join(file)).filter(|file| survey.file(file).is_some());
        let files: Vec<PathBuf> =
            declared.chain(roots).chain(crates).filter(|file| file.starts_with(tree.canonical())).collect();
        files.first().map(|_| ()).map(|()| Target::new(directory, files))
    }

    fn crate_root(survey: &Survey, entry: &Path) -> Option<PathBuf> {
        match survey.kind(entry) {
            Kind::Directory => Some(entry.join(Self::BINARY_ROOT)).filter(|main| survey.file(main).is_some()),
            Kind::File | Kind::Link | Kind::Missing => {
                Some(entry.to_path_buf()).filter(|file| file.extension() == Some(OsStr::new(Self::SOURCE_EXTENSION)))
            }
        }
    }
}
