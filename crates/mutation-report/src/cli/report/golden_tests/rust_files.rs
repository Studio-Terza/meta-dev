use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::report::failure::Failure;

#[derive(Debug)]
pub struct RustFiles;

impl RustFiles {
    const RUST: &'static str = "rs";

    pub fn below(directory: &Path) -> Result<Vec<PathBuf>, Failure> {
        let failed = |source| Failure::Io { path: directory.to_path_buf(), source };
        fs::read_dir(directory).map_err(failed)?.try_fold(Vec::new(), |found, entry| {
            let path = entry.map_err(failed)?.path();
            Ok(found.into_iter().chain(Self::descend(path)?).collect())
        })
    }

    fn descend(path: PathBuf) -> Result<Vec<PathBuf>, Failure> {
        if path.is_dir() {
            Self::below(&path)
        } else {
            Ok(Some(path).filter(|file| Self::is_rust(file)).into_iter().collect())
        }
    }

    fn is_rust(file: &Path) -> bool {
        file.extension().is_some_and(|extension| extension == Self::RUST)
    }
}
