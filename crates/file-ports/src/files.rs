use std::path::{Path, PathBuf};

use crate::{Bytes, FileFailure, Kind, Text};

pub trait Files {
    fn read(&self, path: &Path) -> Result<Bytes, FileFailure>;

    fn read_text(&self, path: &Path) -> Result<Text, FileFailure>;

    fn write(&self, path: &Path, bytes: Bytes) -> Result<(), FileFailure>;

    fn append(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure>;

    fn create_directories(&self, path: &Path) -> Result<(), FileFailure>;

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FileFailure>;

    fn entries(&self, directory: &Path) -> Result<Vec<PathBuf>, FileFailure>;

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure>;

    fn kind(&self, path: &Path) -> Kind;

    fn read_many(&self, paths: &[impl AsRef<Path>]) -> Vec<Result<Bytes, FileFailure>> {
        paths.iter().map(|path| self.read(path.as_ref())).collect()
    }

    fn read_text_many(&self, paths: &[impl AsRef<Path>]) -> Vec<Result<Text, FileFailure>> {
        paths.iter().map(|path| self.read_text(path.as_ref())).collect()
    }

    fn copy_into(&self, paths: &[impl AsRef<Path>], directory: &Path) -> Result<(), FileFailure> {
        paths
            .iter()
            .map(AsRef::as_ref)
            .try_for_each(|path| self.copy(path, &directory.join(path.file_name().unwrap_or_default())))
    }

    fn copy_each_into(&self, paths: &[impl AsRef<Path>], directory: &Path) -> Vec<Result<(), FileFailure>> {
        paths
            .iter()
            .map(AsRef::as_ref)
            .map(|path| self.copy(path, &directory.join(path.file_name().unwrap_or_default())))
            .collect()
    }

    fn entries_many(&self, directories: &[impl AsRef<Path>]) -> Vec<Result<Vec<PathBuf>, FileFailure>> {
        directories.iter().map(|directory| self.entries(directory.as_ref())).collect()
    }

    fn canonical_many(&self, paths: &[impl AsRef<Path>]) -> Vec<Result<PathBuf, FileFailure>> {
        paths.iter().map(|path| self.canonical(path.as_ref())).collect()
    }

    fn kinds(&self, paths: &[impl AsRef<Path>]) -> Vec<Kind> {
        paths.iter().map(|path| self.kind(path.as_ref())).collect()
    }

    fn mirror(&self, root: &Path, into: &impl Files) -> Result<(), FileFailure>
    where
        Self: Sized,
    {
        into.create_directories(root)?;
        self.entries(root)?.iter().try_for_each(|path| {
            Some(path)
                .filter(|path| self.kind(path) == Kind::Directory)
                .map_or_else(|| into.write(path, self.read(path)?), |path| self.mirror(path, into))
        })
    }
}
