use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::{Bytes, FileFailure, Files, Kind, Text};

#[derive(Clone, Copy, Debug, Default)]
pub struct Disk;

impl Files for Disk {
    fn read(&self, path: &Path) -> Result<Bytes, FileFailure> {
        Ok(Bytes::new(fs::read(path)?))
    }

    fn read_text(&self, path: &Path) -> Result<Text, FileFailure> {
        Ok(Text::new(fs::read_to_string(path)?))
    }

    fn write(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        Ok(fs::write(path, bytes)?)
    }

    fn append(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        Ok(OpenOptions::new().append(true).create(true).open(path)?.write_all(bytes.as_ref())?)
    }

    fn create_directories(&self, path: &Path) -> Result<(), FileFailure> {
        Ok(fs::create_dir_all(path)?)
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FileFailure> {
        fs::copy(from, to)?;
        Ok(())
    }

    fn entries(&self, directory: &Path) -> Result<Vec<PathBuf>, FileFailure> {
        Ok(fs::read_dir(directory)?.map(|entry| entry.map(|found| found.path())).collect::<Result<_, _>>()?)
    }

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure> {
        Ok(fs::canonicalize(path)?)
    }

    fn kind(&self, path: &Path) -> Kind {
        fs::symlink_metadata(path).map_or(Kind::Missing, |metadata| match metadata.file_type() {
            found if found.is_dir() => Kind::Directory,
            found if found.is_symlink() => Kind::Link,
            _ => Kind::File,
        })
    }
}
