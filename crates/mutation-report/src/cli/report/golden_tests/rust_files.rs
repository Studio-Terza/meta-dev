use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use file_ports::{FileFailure, Files, Kind};

use crate::cli::report::failure::Failure;

#[derive(Debug)]
pub struct RustFiles;

#[derive(Debug)]
struct Entry {
    path: PathBuf,
    kind: Kind,
}

impl Entry {
    fn listed(
        directory: &Path,
        listing: Result<Vec<PathBuf>, FileFailure>,
        kinds: &BTreeMap<PathBuf, Kind>,
    ) -> Result<Vec<Entry>, Failure> {
        listing
            .map(|paths| Entry::kinded(paths, kinds))
            .map_err(|failure| Failure::Io { path: directory.to_path_buf(), source: io::Error::from(failure) })
    }

    fn kinded(paths: Vec<PathBuf>, kinds: &BTreeMap<PathBuf, Kind>) -> Vec<Entry> {
        paths.into_iter().filter_map(|path| kinds.get(&path).copied().map(|kind| Entry { path, kind })).collect()
    }
}

type Listed = BTreeMap<PathBuf, Result<Vec<Entry>, Failure>>;

impl RustFiles {
    const RUST: &'static str = "rs";

    pub fn below(files: &impl Files, directory: &Path) -> Result<Vec<PathBuf>, Failure> {
        let mut listed = Self::listed(files, vec![directory.to_path_buf()], Listed::new());
        Self::flattened(&mut listed, directory)
    }

    fn listed(files: &impl Files, level: Vec<PathBuf>, found: Listed) -> Listed {
        let entries = files.entries_many(&level);
        let children: Vec<PathBuf> = entries.iter().flatten().flatten().cloned().collect();
        let kinds: BTreeMap<PathBuf, Kind> = children.iter().cloned().zip(files.kinds(&children)).collect();
        let deeper: Vec<PathBuf> =
            kinds.iter().filter(|(_, kind)| **kind == Kind::Directory).map(|(path, _)| path.clone()).collect();
        let found: Listed = found
            .into_iter()
            .chain(level.into_iter().zip(entries).map(|(directory, listing)| {
                let entries = Entry::listed(&directory, listing, &kinds);
                (directory, entries)
            }))
            .collect();
        if deeper.is_empty() { found } else { Self::listed(files, deeper, found) }
    }

    fn flattened(listed: &mut Listed, directory: &Path) -> Result<Vec<PathBuf>, Failure> {
        let entries = listed.remove(directory).transpose()?.unwrap_or_default();
        entries.into_iter().try_fold(Vec::new(), |found, entry| {
            let more = match entry.kind {
                Kind::Directory => Self::flattened(listed, &entry.path)?,
                Kind::File | Kind::Link | Kind::Missing => Self::rust(entry.path).into_iter().collect(),
            };
            Ok(found.into_iter().chain(more).collect())
        })
    }

    fn rust(file: PathBuf) -> Option<PathBuf> {
        Some(file).filter(|file| file.extension().is_some_and(|extension| extension == Self::RUST))
    }
}
