use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use invariants::Ignoring;

mod spread;

use crate::{Bytes, Epoch, FileFailure, Files, Ignored, Kind, Offset, Size, Stamps, Text};
use spread::Spread;

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
        let mut file = File::open(path)?;
        file.seek(SeekFrom::Start(offset.position()))?;
        let mut bytes = Vec::new();
        io::copy(&mut file, &mut bytes)?;
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
        fs::symlink_metadata(path)
            .ignoring(Ignored::Unreadable)
            .map_or(Kind::Missing, |metadata| Kind::from(metadata.file_type()))
    }

    fn size(&self, path: &Path) -> Result<Size, FileFailure> {
        Ok(Size::new(fs::symlink_metadata(path)?.len()))
    }

    fn stamps(&self, path: &Path) -> Result<Stamps, FileFailure> {
        let metadata = fs::metadata(path)?;
        Ok(Stamps::new(Epoch::from(metadata.created()?), Epoch::from(metadata.modified()?)))
    }

    fn read_many(&self, paths: &[impl AsRef<Path>]) -> Vec<Result<Bytes, FileFailure>> {
        Spread::mapped(paths, |path| self.read(path))
    }

    fn entries_many(&self, directories: &[impl AsRef<Path>]) -> Vec<Result<Vec<PathBuf>, FileFailure>> {
        Spread::mapped(directories, |directory| self.entries(directory))
    }

    fn canonical_many(&self, paths: &[impl AsRef<Path>]) -> Vec<Result<PathBuf, FileFailure>> {
        Spread::mapped(paths, |path| self.canonical(path))
    }

    fn kinds(&self, paths: &[impl AsRef<Path>]) -> Vec<Kind> {
        Spread::mapped(paths, |path| self.kind(path))
    }
}
