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
}
