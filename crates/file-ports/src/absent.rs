use std::io;
use std::path::{Path, PathBuf};

use crate::{Bytes, FileFailure, Files, Kind, Size, Text};

#[derive(Clone, Copy, Debug, Default)]
pub struct Absent;

impl Absent {
    fn missing() -> FileFailure {
        FileFailure::Missing(io::Error::from(io::ErrorKind::NotFound))
    }
}

impl Files for Absent {
    fn read(&self, _path: &Path) -> Result<Bytes, FileFailure> {
        Err(Self::missing())
    }

    fn read_text(&self, _path: &Path) -> Result<Text, FileFailure> {
        Err(Self::missing())
    }

    fn write(&self, _path: &Path, _bytes: Bytes) -> Result<(), FileFailure> {
        Err(Self::missing())
    }

    fn append(&self, _path: &Path, _bytes: &Bytes) -> Result<(), FileFailure> {
        Err(Self::missing())
    }

    fn create_directories(&self, _path: &Path) -> Result<(), FileFailure> {
        Err(Self::missing())
    }

    fn copy(&self, _from: &Path, _to: &Path) -> Result<(), FileFailure> {
        Err(Self::missing())
    }

    fn entries(&self, _directory: &Path) -> Result<Vec<PathBuf>, FileFailure> {
        Err(Self::missing())
    }

    fn canonical(&self, _path: &Path) -> Result<PathBuf, FileFailure> {
        Err(Self::missing())
    }

    fn kind(&self, _path: &Path) -> Kind {
        Kind::Missing
    }

    fn size(&self, _path: &Path) -> Result<Size, FileFailure> {
        Err(Self::missing())
    }
}
