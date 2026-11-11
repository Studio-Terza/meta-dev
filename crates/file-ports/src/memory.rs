mod locked;
mod resolved;

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::ops::Bound;
use std::path::{Path, PathBuf};

use newtype::Wrapping;

use crate::{Bytes, FileFailure, Files, Kind, Size, Text};
use locked::Locked;
use resolved::Resolved;

#[derive(Debug, Default)]
pub struct Memory {
    files: Locked<BTreeMap<PathBuf, Bytes>>,
    directories: Locked<BTreeSet<PathBuf>>,
}

impl Memory {
    fn missing() -> FileFailure {
        FileFailure::Missing(io::Error::from(io::ErrorKind::NotFound))
    }

    fn stored(&self, path: &Path) -> Result<Bytes, FileFailure> {
        let found = self.files.held().get(path).cloned();
        found.ok_or_else(|| {
            Some(self.kind(path))
                .filter(|kind| *kind == Kind::Directory)
                .map_or_else(Self::missing, |_| FileFailure::Failed(io::Error::from(io::ErrorKind::IsADirectory)))
        })
    }

    fn placed<'path>(&self, path: &'path Path) -> Result<&'path Path, FileFailure> {
        path.parent()
            .filter(|parent| self.kind(parent) != Kind::Directory)
            .filter(|parent| parent.components().next().is_some())
            .map_or(Ok(path), |_| Err(Self::missing()))
    }

    fn holds_below(&self, path: &Path) -> Option<Kind> {
        let files = self.files.held();
        let directories = self.directories.held();
        files
            .keys()
            .chain(directories.iter())
            .filter(|held| *held != path)
            .find(|held| held.starts_with(path))
            .map(|_| Kind::Directory)
    }
}

impl FromIterator<(PathBuf, Bytes)> for Memory {
    fn from_iter<Entries: IntoIterator<Item = (PathBuf, Bytes)>>(entries: Entries) -> Memory {
        Memory { files: Locked::new(entries.into_iter().collect()), directories: Locked::default() }
    }
}

impl Files for Memory {
    fn read(&self, path: &Path) -> Result<Bytes, FileFailure> {
        self.stored(path)
    }

    fn read_text(&self, path: &Path) -> Result<Text, FileFailure> {
        Text::try_from(&self.stored(path)?)
    }

    fn write(&self, path: &Path, bytes: Bytes) -> Result<(), FileFailure> {
        let placed = self.placed(path)?.to_path_buf();
        self.files.held().insert(placed, bytes);
        Ok(())
    }

    fn append(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        let held = self.stored(path).unwrap_or_default().into_inner();
        let joined = held.into_iter().chain(bytes.as_ref().iter().copied()).collect();
        self.write(path, Bytes::new(joined))
    }

    fn create_directories(&self, path: &Path) -> Result<(), FileFailure> {
        let blocked = path.ancestors().find(|ancestor| self.files.held().contains_key(*ancestor));
        blocked.map_or_else(
            || {
                self.directories.held().insert(path.to_path_buf());
                Ok(())
            },
            |_| Err(FileFailure::Failed(io::Error::from(io::ErrorKind::NotADirectory))),
        )
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FileFailure> {
        self.write(to, self.stored(from)?)
    }

    fn entries(&self, directory: &Path) -> Result<Vec<PathBuf>, FileFailure> {
        Some(self.kind(directory)).filter(|kind| *kind == Kind::Directory).ok_or_else(Self::missing).map(|_| {
            let files = self.files.held();
            let directories = self.directories.held();
            let children: BTreeSet<PathBuf> = files
                .range::<Path, _>((Bound::Included(directory), Bound::Unbounded))
                .map(|(held, _bytes)| held)
                .take_while(|held| held.starts_with(directory))
                .chain(
                    directories
                        .range::<Path, _>((Bound::Included(directory), Bound::Unbounded))
                        .take_while(|held| held.starts_with(directory)),
                )
                .filter_map(|held| held.strip_prefix(directory).ok())
                .filter_map(|inside| inside.components().next())
                .map(|first| directory.join(first))
                .collect();
            children.into_iter().collect()
        })
    }

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure> {
        Some(Resolved::from(path).into_inner())
            .filter(|resolved| self.kind(resolved) != Kind::Missing)
            .ok_or_else(Self::missing)
    }

    fn kind(&self, path: &Path) -> Kind {
        let file = self.files.held().contains_key(path).then_some(Kind::File);
        let directory = self.directories.held().contains(path).then_some(Kind::Directory);
        file.or(directory).or_else(|| self.holds_below(path)).unwrap_or(Kind::Missing)
    }

    fn size(&self, path: &Path) -> Result<Size, FileFailure> {
        self.stored(path).map(|bytes| Size::from(&bytes))
    }
}
