use std::io;
use std::path::{Path, PathBuf};

use file_ports::{Files, Kind};

use crate::cli::report::failure::Failure;

#[derive(Debug)]
pub struct RustFiles;

impl RustFiles {
    const RUST: &'static str = "rs";

    pub fn below(files: &impl Files, directory: &Path) -> Result<Vec<PathBuf>, Failure> {
        let entries = files
            .entries(directory)
            .map_err(|failure| Failure::Io { path: directory.to_path_buf(), source: io::Error::from(failure) })?;
        entries
            .into_iter()
            .try_fold(Vec::new(), |found, path| Ok(found.into_iter().chain(Self::descend(files, path)?).collect()))
    }

    fn descend(files: &impl Files, path: PathBuf) -> Result<Vec<PathBuf>, Failure> {
        match files.kind(&path) {
            Kind::Directory => Self::below(files, &path),
            _ => Ok(Self::rust(path).into_iter().collect()),
        }
    }

    fn rust(file: PathBuf) -> Option<PathBuf> {
        Some(file).filter(|file| file.extension().is_some_and(|extension| extension == Self::RUST))
    }
}
