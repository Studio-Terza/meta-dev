use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use file_ports::{Bytes, FileFailure, Files, Kind, Memory, Text};

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
        if directory == self.unlistable {
            Err(FileFailure::Failed(io::Error::from(io::ErrorKind::PermissionDenied)))
        } else {
            self.memory.entries(directory)
        }
    }

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure> {
        self.memory.canonical(path)
    }

    fn kind(&self, path: &Path) -> Kind {
        self.memory.kind(path)
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
        if path == self.link { &self.target } else { path }
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
        if path == self.link { Kind::Link } else { self.memory.kind(path) }
    }
}

pub struct FakeFlaky<'memory> {
    memory: &'memory Memory,
    listings: AtomicUsize,
}

impl FakeFlaky<'_> {
    pub fn new(memory: &Memory) -> FakeFlaky<'_> {
        FakeFlaky { memory, listings: AtomicUsize::new(0) }
    }
}

impl Files for FakeFlaky<'_> {
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
        let earlier = self.listings.fetch_add(1, Ordering::SeqCst);
        if earlier == 0 {
            self.memory.entries(directory)
        } else {
            Err(FileFailure::Failed(io::Error::from(io::ErrorKind::PermissionDenied)))
        }
    }

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure> {
        self.memory.canonical(path)
    }

    fn kind(&self, path: &Path) -> Kind {
        self.memory.kind(path)
    }
}
