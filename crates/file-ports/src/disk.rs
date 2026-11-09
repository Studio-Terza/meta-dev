use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

use newtype::Wrapping;

use crate::{Bytes, FileFailure, Files, Kind, Offset, Size, Text};

#[derive(Clone, Copy, Debug, Default)]
pub struct Disk;

impl Files for Disk {
    fn read(&self, path: &Path) -> Result<Bytes, FileFailure> {
        Ok(Bytes::new(fs::read(path)?))
    }

    fn read_text(&self, path: &Path) -> Result<Text, FileFailure> {
        Ok(Text::new(fs::read_to_string(path)?))
    }

    fn read_from(&self, path: &Path, offset: Offset) -> Result<Bytes, FileFailure> {
        let file = File::open(path)?;
        let start = offset.into_inner();
        let remaining = usize::try_from(file.metadata()?.len().saturating_sub(start)).map_err(io::Error::other)?;
        let mut bytes = vec![0_u8; remaining];
        file.read_exact_at(&mut bytes, start)?;
        Ok(Bytes::new(bytes))
    }

    fn write(&self, path: &Path, bytes: Bytes) -> Result<(), FileFailure> {
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
        fs::symlink_metadata(path).map_or(Kind::Missing, |metadata| Kind::from(metadata.file_type()))
    }

    fn size(&self, path: &Path) -> Result<Size, FileFailure> {
        Ok(Size::new(fs::symlink_metadata(path)?.len()))
    }
}
