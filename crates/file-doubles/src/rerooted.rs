use std::path::{Path, PathBuf};

use file_ports::{Bytes, Disk, FileFailure, FilesFiller, Kind, Offset, Size, Stamps, Text, files_ops};
use invariants::Ignoring;
use port_core::Pass;
use sqlite_ports::SqliteOpener;
use store_ports::{Opens, Schema};

use crate::Ignored;

#[derive(Clone, Debug)]
pub struct Rerooted {
    moved: Vec<Moved>,
}

#[derive(Clone, Debug)]
struct Moved {
    from: PathBuf,
    to: PathBuf,
}

impl FromIterator<(PathBuf, PathBuf)> for Rerooted {
    fn from_iter<Moves: IntoIterator<Item = (PathBuf, PathBuf)>>(moves: Moves) -> Rerooted {
        Rerooted { moved: moves.into_iter().map(|(from, to)| Moved { from, to }).collect() }
    }
}

impl Opens for Rerooted {
    type Store = <SqliteOpener as Opens>::Store;

    fn file<S: Schema>(&self, path: PathBuf) -> Self::Store {
        SqliteOpener.file::<S>(self.real(&path))
    }

    fn in_memory<S: Schema>(&self) -> Self::Store {
        SqliteOpener.in_memory::<S>()
    }
}

impl Rerooted {
    #[must_use]
    pub fn real(&self, path: &Path) -> PathBuf {
        self.moved
            .iter()
            .find_map(|moved| {
                path.strip_prefix(&moved.from).ignoring(Ignored::Outside).map(|below| moved.to.join(below))
            })
            .unwrap_or_else(|| path.to_path_buf())
    }
}

impl FilesFiller for Rerooted {
    fn read(&self, pass: Pass<'_, files_ops::Read>, path: &Path) -> Result<Bytes, FileFailure> {
        Disk.read(pass, &self.real(path))
    }

    fn read_text(&self, pass: Pass<'_, files_ops::ReadText>, path: &Path) -> Result<Text, FileFailure> {
        Disk.read_text(pass, &self.real(path))
    }

    fn read_from(
        &self,
        pass: Pass<'_, files_ops::ReadFrom>,
        path: &Path,
        offset: Offset,
    ) -> Result<Bytes, FileFailure> {
        Disk.read_from(pass, &self.real(path), offset)
    }

    fn write(&self, pass: Pass<'_, files_ops::Write>, path: &Path, bytes: Bytes) -> Result<(), FileFailure> {
        Disk.write(pass, &self.real(path), bytes)
    }

    fn append(&self, pass: Pass<'_, files_ops::Append>, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        Disk.append(pass, &self.real(path), bytes)
    }

    fn create_directories(&self, pass: Pass<'_, files_ops::CreateDirectories>, path: &Path) -> Result<(), FileFailure> {
        Disk.create_directories(pass, &self.real(path))
    }

    fn entries(&self, pass: Pass<'_, files_ops::Entries>, directory: &Path) -> Result<Vec<PathBuf>, FileFailure> {
        Disk.entries(pass, &self.real(directory))
    }

    fn canonical(&self, pass: Pass<'_, files_ops::Canonical>, path: &Path) -> Result<PathBuf, FileFailure> {
        Disk.canonical(pass, &self.real(path))
    }

    fn kind(&self, pass: Pass<'_, files_ops::Kind>, path: &Path) -> Kind {
        Disk.kind(pass, &self.real(path))
    }

    fn size(&self, pass: Pass<'_, files_ops::Size>, path: &Path) -> Result<Size, FileFailure> {
        Disk.size(pass, &self.real(path))
    }

    fn stamps(&self, pass: Pass<'_, files_ops::Stamps>, path: &Path) -> Result<Stamps, FileFailure> {
        Disk.stamps(pass, &self.real(path))
    }
}
