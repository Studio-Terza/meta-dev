use std::path::{Path, PathBuf};

use crate::{Bytes, FileFailure, Kind, Text};

pub trait Files {
    fn read(&self, path: &Path) -> Result<Bytes, FileFailure>;

    fn read_text(&self, path: &Path) -> Result<Text, FileFailure>;

    fn write(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure>;

    fn append(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure>;

    fn create_directories(&self, path: &Path) -> Result<(), FileFailure>;

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FileFailure>;

    fn entries(&self, directory: &Path) -> Result<Vec<PathBuf>, FileFailure>;

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure>;

    fn kind(&self, path: &Path) -> Kind;

    fn read_many(&self, paths: &[PathBuf]) -> Vec<Result<Bytes, FileFailure>> {
        paths.iter().map(|path| self.read(path)).collect()
    }

    fn read_text_many(&self, paths: &[PathBuf]) -> Vec<Result<Text, FileFailure>> {
        paths.iter().map(|path| self.read_text(path)).collect()
    }

    fn copy_into(&self, paths: &[PathBuf], directory: &Path) -> Result<(), FileFailure> {
        paths.iter().try_for_each(|path| self.copy(path, &directory.join(path.file_name().unwrap_or_default())))
    }

    fn entries_many(&self, directories: &[PathBuf]) -> Vec<Result<Vec<PathBuf>, FileFailure>> {
        directories.iter().map(|directory| self.entries(directory)).collect()
    }

    fn canonical_many(&self, paths: &[PathBuf]) -> Vec<Result<PathBuf, FileFailure>> {
        paths.iter().map(|path| self.canonical(path)).collect()
    }

    fn kinds(&self, paths: &[PathBuf]) -> Vec<Kind> {
        paths.iter().map(|path| self.kind(path)).collect()
    }
}
