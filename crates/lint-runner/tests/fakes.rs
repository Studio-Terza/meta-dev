use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use file_ports::{Bytes, FileFailure, Files, Kind, Memory, Size, Text};

pub struct FakeUnlistable<'memory> {
    memory: &'memory Memory,
    unlistable: PathBuf,
}

impl FakeUnlistable<'_> {
    pub fn new(memory: &Memory, unlistable: PathBuf) -> FakeUnlistable<'_> {
        FakeUnlistable { memory, unlistable }
    }
}

impl Files for FakeUnlistable<'_> {
    fn read(&self, path: &Path) -> Result<Bytes, FileFailure> {
        self.memory.read(path)
    }

    fn read_text(&self, path: &Path) -> Result<Text, FileFailure> {
        self.memory.read_text(path)
    }

    fn write(&self, path: &Path, bytes: Bytes) -> Result<(), FileFailure> {
        self.memory.write(path, bytes)
    }

    fn append(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        self.memory.append(path, bytes)
    }

    fn create_directories(&self, path: &Path) -> Result<(), FileFailure> {
        self.memory.create_directories(path)
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FileFailure> {
        self.memory.copy(from, to)
    }

    fn entries(&self, directory: &Path) -> Result<Vec<PathBuf>, FileFailure> {
        Some(directory).filter(|candidate| *candidate == self.unlistable).map_or_else(
            || self.memory.entries(directory),
            |_| Err(FileFailure::Failed(io::Error::from(io::ErrorKind::PermissionDenied))),
        )
    }

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure> {
        self.memory.canonical(path)
    }

    fn kind(&self, path: &Path) -> Kind {
        self.memory.kind(path)
    }

    fn size(&self, path: &Path) -> Result<Size, FileFailure> {
        self.memory.size(path)
    }
}

pub struct FakeLinked<'memory> {
    memory: &'memory Memory,
    link: PathBuf,
    target: PathBuf,
}

impl FakeLinked<'_> {
    pub fn new(memory: &Memory, link: PathBuf, target: PathBuf) -> FakeLinked<'_> {
        FakeLinked { memory, link, target }
    }

    fn followed<'path>(&'path self, path: &'path Path) -> &'path Path {
        Some(path).filter(|candidate| *candidate == self.link).map_or(path, |_| &self.target)
    }
}

impl Files for FakeLinked<'_> {
    fn read(&self, path: &Path) -> Result<Bytes, FileFailure> {
        self.memory.read(self.followed(path))
    }

    fn read_text(&self, path: &Path) -> Result<Text, FileFailure> {
        self.memory.read_text(self.followed(path))
    }

    fn write(&self, path: &Path, bytes: Bytes) -> Result<(), FileFailure> {
        self.memory.write(path, bytes)
    }

    fn append(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        self.memory.append(path, bytes)
    }

    fn create_directories(&self, path: &Path) -> Result<(), FileFailure> {
        self.memory.create_directories(path)
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FileFailure> {
        self.memory.copy(from, to)
    }

    fn entries(&self, directory: &Path) -> Result<Vec<PathBuf>, FileFailure> {
        let mut entries = self.memory.entries(directory)?;
        entries.extend(self.link.parent().filter(|parent| *parent == directory).map(|_| self.link.clone()));
        Ok(entries)
    }

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure> {
        self.memory.canonical(self.followed(path))
    }

    fn kind(&self, path: &Path) -> Kind {
        Some(path).filter(|candidate| *candidate == self.link).map_or_else(|| self.memory.kind(path), |_| Kind::Link)
    }

    fn size(&self, path: &Path) -> Result<Size, FileFailure> {
        self.memory.size(path)
    }
}

const SOURCE_EXTENSION: &str = "rs";

pub struct FakeCounted<'memory> {
    memory: &'memory Memory,
    counted: PathBuf,
    listings: AtomicUsize,
    sources: AtomicUsize,
}

impl FakeCounted<'_> {
    pub fn new(memory: &Memory, counted: PathBuf) -> FakeCounted<'_> {
        FakeCounted { memory, counted, listings: AtomicUsize::new(0), sources: AtomicUsize::new(0) }
    }

    pub fn listings(&self) -> usize {
        self.listings.load(Ordering::SeqCst)
    }

    pub fn sources(&self) -> usize {
        self.sources.load(Ordering::SeqCst)
    }
}

impl Files for FakeCounted<'_> {
    fn read(&self, path: &Path) -> Result<Bytes, FileFailure> {
        self.memory.read(path)
    }

    fn read_text(&self, path: &Path) -> Result<Text, FileFailure> {
        self.sources.fetch_add(usize::from(path.extension() == Some(OsStr::new(SOURCE_EXTENSION))), Ordering::SeqCst);
        self.memory.read_text(path)
    }

    fn write(&self, path: &Path, bytes: Bytes) -> Result<(), FileFailure> {
        self.memory.write(path, bytes)
    }

    fn append(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        self.memory.append(path, bytes)
    }

    fn create_directories(&self, path: &Path) -> Result<(), FileFailure> {
        self.memory.create_directories(path)
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FileFailure> {
        self.memory.copy(from, to)
    }

    fn entries(&self, directory: &Path) -> Result<Vec<PathBuf>, FileFailure> {
        let _listed = (directory == self.counted).then(|| self.listings.fetch_add(1, Ordering::SeqCst));
        self.memory.entries(directory)
    }

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure> {
        self.memory.canonical(path)
    }

    fn kind(&self, path: &Path) -> Kind {
        self.memory.kind(path)
    }

    fn size(&self, path: &Path) -> Result<Size, FileFailure> {
        self.memory.size(path)
    }
}
