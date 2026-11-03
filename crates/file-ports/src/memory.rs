use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

use newtype::Newtype;

use crate::{Bytes, FileFailure, Files, Kind, Text};

#[derive(Debug, Default)]
pub struct Memory {
    files: RefCell<BTreeMap<PathBuf, Bytes>>,
    directories: RefCell<BTreeSet<PathBuf>>,
}

impl Memory {
    fn missing() -> FileFailure {
        FileFailure::Missing(io::Error::from(io::ErrorKind::NotFound))
    }

    fn stored(&self, path: &Path) -> Result<Bytes, FileFailure> {
        self.files.borrow().get(path).cloned().ok_or_else(Self::missing)
    }

    fn placed<'path>(&self, path: &'path Path) -> Result<&'path Path, FileFailure> {
        match path.parent().map(|parent| (parent, self.kind(parent))) {
            Some((_, Kind::Directory)) | None => Ok(path),
            Some((parent, _)) if parent.as_os_str().is_empty() => Ok(path),
            Some(_) => Err(Self::missing()),
        }
    }

    fn holds_below(&self, path: &Path) -> Option<Kind> {
        let files = self.files.borrow();
        let directories = self.directories.borrow();
        files
            .keys()
            .chain(directories.iter())
            .find(|held| *held != path && held.starts_with(path))
            .map(|_| Kind::Directory)
    }
}

impl FromIterator<(PathBuf, Bytes)> for Memory {
    fn from_iter<Entries: IntoIterator<Item = (PathBuf, Bytes)>>(entries: Entries) -> Memory {
        Memory { files: RefCell::new(entries.into_iter().collect()), directories: RefCell::default() }
    }
}

impl Files for Memory {
    fn read(&self, path: &Path) -> Result<Bytes, FileFailure> {
        self.stored(path)
    }

    fn read_text(&self, path: &Path) -> Result<Text, FileFailure> {
        String::from_utf8(self.stored(path)?.into_inner())
            .map(Text::new)
            .map_err(|_| FileFailure::Failed(io::Error::from(io::ErrorKind::InvalidData)))
    }

    fn write(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        let placed = self.placed(path)?.to_path_buf();
        self.files.borrow_mut().insert(placed, bytes.clone());
        Ok(())
    }

    fn append(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        let held = self.stored(path).unwrap_or_default().into_inner();
        let joined = held.into_iter().chain(bytes.as_ref().iter().copied()).collect();
        self.write(path, &Bytes::new(joined))
    }

    fn create_directories(&self, path: &Path) -> Result<(), FileFailure> {
        if path.ancestors().any(|ancestor| self.files.borrow().contains_key(ancestor)) {
            Err(FileFailure::Failed(io::Error::from(io::ErrorKind::NotADirectory)))
        } else {
            self.directories.borrow_mut().insert(path.to_path_buf());
            Ok(())
        }
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FileFailure> {
        self.write(to, &self.stored(from)?)
    }

    fn entries(&self, directory: &Path) -> Result<Vec<PathBuf>, FileFailure> {
        match self.kind(directory) {
            Kind::Directory => {
                let files = self.files.borrow();
                let directories = self.directories.borrow();
                let children: BTreeSet<PathBuf> = files
                    .keys()
                    .chain(directories.iter())
                    .filter_map(|held| held.strip_prefix(directory).ok())
                    .filter_map(|inside| inside.components().next())
                    .map(|first| directory.join(first))
                    .collect();
                Ok(children.into_iter().collect())
            }
            _ => Err(Self::missing()),
        }
    }

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure> {
        match self.kind(path) {
            Kind::Missing => Err(Self::missing()),
            _ => Ok(path.to_path_buf()),
        }
    }

    fn kind(&self, path: &Path) -> Kind {
        let file = self.files.borrow().contains_key(path).then_some(Kind::File);
        let directory = self.directories.borrow().contains(path).then_some(Kind::Directory);
        file.or(directory).or_else(|| self.holds_below(path)).unwrap_or(Kind::Missing)
    }
}
