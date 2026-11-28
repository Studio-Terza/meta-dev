use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use invariants::Ignoring;
use port_core::Pass;

use crate::ignored::Ignored;
use crate::{Bytes, Epoch, FileFailure, FilesFiller, Kind, Offset, Size, Stamps, Text, files_ops};

#[derive(Clone, Copy, Debug, Default)]
pub struct Disk;

impl FilesFiller for Disk {
    fn read(&self, _pass: Pass<'_, files_ops::Read>, path: &Path) -> Result<Bytes, FileFailure> {
        Ok(Bytes::new(fs::read(path)?))
    }

    fn read_text(&self, _pass: Pass<'_, files_ops::ReadText>, path: &Path) -> Result<Text, FileFailure> {
        Ok(Text::new(fs::read_to_string(path)?))
    }

    fn read_from(
        &self,
        _pass: Pass<'_, files_ops::ReadFrom>,
        path: &Path,
        offset: Offset,
    ) -> Result<Bytes, FileFailure> {
        let mut file = File::open(path)?;
        file.seek(SeekFrom::Start(offset.position()))?;
        let mut bytes = Vec::new();
        io::copy(&mut file, &mut bytes)?;
        Ok(Bytes::new(bytes))
    }

    fn write(&self, _pass: Pass<'_, files_ops::Write>, path: &Path, bytes: Bytes) -> Result<(), FileFailure> {
        Ok(fs::write(path, bytes)?)
    }

    fn append(&self, _pass: Pass<'_, files_ops::Append>, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        Ok(OpenOptions::new().append(true).create(true).open(path)?.write_all(bytes.as_ref())?)
    }

    fn remove(&self, _pass: Pass<'_, files_ops::Remove>, path: &Path) -> Result<(), FileFailure> {
        Ok(fs::remove_file(path)?)
    }

    fn create_directories(
        &self,
        _pass: Pass<'_, files_ops::CreateDirectories>,
        path: &Path,
    ) -> Result<(), FileFailure> {
        Ok(fs::create_dir_all(path)?)
    }

    fn entries(&self, _pass: Pass<'_, files_ops::Entries>, directory: &Path) -> Result<Vec<PathBuf>, FileFailure> {
        let listed: BTreeSet<PathBuf> =
            fs::read_dir(directory)?.map(|entry| entry.map(|found| found.path())).collect::<Result<_, _>>()?;
        Ok(listed.into_iter().collect())
    }

    fn canonical(&self, _pass: Pass<'_, files_ops::Canonical>, path: &Path) -> Result<PathBuf, FileFailure> {
        Ok(fs::canonicalize(path)?)
    }

    fn kind(&self, _pass: Pass<'_, files_ops::Kind>, path: &Path) -> Kind {
        fs::symlink_metadata(path)
            .ignoring(Ignored::Unreadable)
            .map_or(Kind::Missing, |metadata| Kind::from(metadata.file_type()))
    }

    fn size(&self, _pass: Pass<'_, files_ops::Size>, path: &Path) -> Result<Size, FileFailure> {
        Ok(Size::new(fs::symlink_metadata(path)?.len()))
    }

    fn stamps(&self, _pass: Pass<'_, files_ops::Stamps>, path: &Path) -> Result<Stamps, FileFailure> {
        let metadata = fs::metadata(path)?;
        Ok(Stamps::new(Epoch::from(metadata.created()?), Epoch::from(metadata.modified()?)))
    }
}
