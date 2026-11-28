mod locked;
mod resolved;

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::ops::Bound;
use std::path::{Path, PathBuf};

use invariants::Ignoring;
use newtype::Wrapping;
use newtype::serde::{Deserialize, Deserializer};
use port_core::Pass;

use file_ports::{Bytes, FileFailure, FilesFiller, Kind, Offset, Size, Stamps, Text, files_ops};

use crate::ignored::Ignored;
use locked::Locked;
use resolved::Resolved;

#[derive(Debug, Default)]
pub struct Memory {
    files: Locked<BTreeMap<PathBuf, Bytes>>,
    directories: Locked<BTreeSet<PathBuf>>,
}

impl Memory {
    fn missing() -> FileFailure {
        FileFailure::from(io::Error::from(io::ErrorKind::NotFound))
    }

    fn stored(&self, path: &Path) -> Result<Bytes, FileFailure> {
        self.files.held().get(path).cloned().ok_or_else(Self::missing)
    }

    fn placed<'path>(&self, path: &'path Path) -> Result<&'path Path, FileFailure> {
        path.parent()
            .filter(|parent| self.found(parent) != Kind::Directory)
            .filter(|parent| parent.components().next().is_some())
            .map_or(Ok(path), |_| Err(Self::missing()))
    }

    fn put(&self, path: &Path, bytes: Bytes) -> Result<(), FileFailure> {
        let placed = self.placed(path)?.to_path_buf();
        self.files.held().insert(placed, bytes);
        Ok(())
    }

    fn found(&self, path: &Path) -> Kind {
        let files = self.files.held();
        let directories = self.directories.held();
        let file = files.contains_key(path).then_some(Kind::File);
        let below = || {
            files
                .keys()
                .chain(directories.iter())
                .find(|held| held.starts_with(path) && (*held != path || directories.contains(path)))
        };
        file.or_else(|| below().map(|_| Kind::Directory)).unwrap_or(Kind::Missing)
    }
}

impl<'de> Deserialize<'de> for Memory {
    fn deserialize<Reader: Deserializer<'de>>(reader: Reader) -> Result<Memory, Reader::Error> {
        let texts = BTreeMap::<PathBuf, Text>::deserialize(reader)?;
        Ok(texts.into_iter().map(|(path, text)| (path, Bytes::from(&text))).collect())
    }
}

impl FromIterator<(PathBuf, Bytes)> for Memory {
    fn from_iter<Entries: IntoIterator<Item = (PathBuf, Bytes)>>(entries: Entries) -> Memory {
        Memory { files: Locked::new(entries.into_iter().collect()), directories: Locked::default() }
    }
}

impl FilesFiller for Memory {
    fn read(&self, _pass: Pass<'_, files_ops::Read>, path: &Path) -> Result<Bytes, FileFailure> {
        self.stored(path)
    }

    fn read_text(&self, _pass: Pass<'_, files_ops::ReadText>, path: &Path) -> Result<Text, FileFailure> {
        Text::try_from(&self.stored(path)?)
    }

    fn read_from(
        &self,
        _pass: Pass<'_, files_ops::ReadFrom>,
        path: &Path,
        offset: Offset,
    ) -> Result<Bytes, FileFailure> {
        let skipped = usize::try_from(offset.position()).ignoring(Ignored::BeyondRange).unwrap_or(usize::MAX);
        self.stored(path).map(|bytes| Bytes::new(bytes.as_ref().iter().skip(skipped).copied().collect()))
    }

    fn write(&self, _pass: Pass<'_, files_ops::Write>, path: &Path, bytes: Bytes) -> Result<(), FileFailure> {
        self.put(path, bytes)
    }

    fn append(&self, _pass: Pass<'_, files_ops::Append>, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        let held = self.stored(path).ignoring(Ignored::Unstored).unwrap_or_default().into_inner();
        let joined = held.into_iter().chain(bytes.as_ref().iter().copied()).collect();
        self.put(path, Bytes::new(joined))
    }

    fn remove(&self, _pass: Pass<'_, files_ops::Remove>, path: &Path) -> Result<(), FileFailure> {
        self.files.held().remove(path).map(|_removed| ()).ok_or_else(Self::missing)
    }

    fn create_directories(
        &self,
        _pass: Pass<'_, files_ops::CreateDirectories>,
        path: &Path,
    ) -> Result<(), FileFailure> {
        let blocked = path.ancestors().any(|ancestor| self.files.held().contains_key(ancestor));
        Some(())
            .filter(|()| !blocked)
            .ok_or_else(|| FileFailure::from(io::Error::from(io::ErrorKind::NotADirectory)))?;
        self.directories.held().insert(path.to_path_buf());
        Ok(())
    }

    fn entries(&self, _pass: Pass<'_, files_ops::Entries>, directory: &Path) -> Result<Vec<PathBuf>, FileFailure> {
        Some(self.found(directory)).filter(|kind| *kind == Kind::Directory).ok_or_else(Self::missing).map(|_| {
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
                .filter_map(|held| held.strip_prefix(directory).ignoring(Ignored::Outside))
                .filter_map(|inside| inside.components().next())
                .map(|first| directory.join(first))
                .collect();
            children.into_iter().collect()
        })
    }

    fn canonical(&self, _pass: Pass<'_, files_ops::Canonical>, path: &Path) -> Result<PathBuf, FileFailure> {
        Some(Resolved::from(path).into_inner())
            .filter(|resolved| self.found(resolved) != Kind::Missing)
            .ok_or_else(Self::missing)
    }

    fn kind(&self, _pass: Pass<'_, files_ops::Kind>, path: &Path) -> Kind {
        self.found(path)
    }

    fn size(&self, _pass: Pass<'_, files_ops::Size>, path: &Path) -> Result<Size, FileFailure> {
        self.stored(path).map(|bytes| Size::from(&bytes))
    }

    fn stamps(&self, _pass: Pass<'_, files_ops::Stamps>, path: &Path) -> Result<Stamps, FileFailure> {
        self.stored(path)?;
        Err(FileFailure::from(io::Error::from(io::ErrorKind::Unsupported)))
    }
}
