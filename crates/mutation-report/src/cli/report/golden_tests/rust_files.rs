use std::borrow::Cow;
use std::io;
use std::path::{Path, PathBuf};

use file_ports::{FileFailure, Files, Kind};

use crate::cli::report::failure::Failure;

#[derive(Debug)]
pub struct RustFiles;

#[derive(Debug)]
enum Entry {
    Rust(PathBuf),
    Directory(PathBuf),
}

type Listing = Result<Vec<PathBuf>, Failure<'static>>;

impl Entry {
    fn kinded(path: PathBuf, kind: Kind) -> Option<Entry> {
        match kind {
            Kind::Directory => Some(Entry::Directory(path)),
            Kind::File | Kind::Link | Kind::Missing => RustFiles::rust(path).map(Entry::Rust),
        }
    }

    fn listed(
        directory: &Path,
        listing: Result<Vec<PathBuf>, FileFailure>,
        kinds: &mut impl Iterator<Item = Kind>,
    ) -> Result<Vec<Entry>, Failure<'static>> {
        listing
            .map(|paths| paths.into_iter().zip(kinds).filter_map(|(path, kind)| Entry::kinded(path, kind)).collect())
            .map_err(|failure| Failure::Io {
                path: Cow::Owned(directory.to_path_buf()),
                source: io::Error::from(failure),
            })
    }

    fn directory(&self) -> Option<&Path> {
        match self {
            Entry::Directory(path) => Some(path),
            Entry::Rust(_) => None,
        }
    }

    fn found(self, below: &mut impl Iterator<Item = Listing>) -> Listing {
        match self {
            Entry::Rust(path) => Ok(vec![path]),
            Entry::Directory(_) => below.next().unwrap_or_else(|| Ok(Vec::new())),
        }
    }
}

impl RustFiles {
    const RUST: &'static str = "rs";

    pub fn below(files: &impl Files, directory: &Path) -> Listing {
        Self::listed(files, &[directory]).into_iter().next().unwrap_or_else(|| Ok(Vec::new()))
    }

    fn listed(files: &impl Files, level: &[&Path]) -> Vec<Listing> {
        let listings = files.entries_many(level);
        let kinds = files.kinds(&listings.iter().flatten().flatten().collect::<Vec<&PathBuf>>());
        let entries: Vec<Result<Vec<Entry>, Failure<'static>>> = level
            .iter()
            .zip(listings)
            .scan(kinds.into_iter(), |remaining, (directory, listing)| {
                Some(Entry::listed(directory, listing, remaining))
            })
            .collect();
        let deeper: Vec<&Path> = entries.iter().flatten().flatten().filter_map(Entry::directory).collect();
        let below = deeper.first().map_or_else(Vec::new, |_| Self::listed(files, &deeper));
        entries
            .into_iter()
            .scan(below.into_iter(), |remaining, listed| {
                Some(listed.and_then(|found| Self::flattened(found, remaining)))
            })
            .collect()
    }

    fn flattened(entries: Vec<Entry>, below: &mut impl Iterator<Item = Listing>) -> Listing {
        let found: Vec<Listing> = entries.into_iter().map(|entry| entry.found(below)).collect();
        found.into_iter().try_fold(Vec::new(), |gathered, more| Ok(gathered.into_iter().chain(more?).collect()))
    }

    fn rust(file: PathBuf) -> Option<PathBuf> {
        Some(file).filter(|candidate| candidate.extension().is_some_and(|extension| extension == Self::RUST))
    }
}
