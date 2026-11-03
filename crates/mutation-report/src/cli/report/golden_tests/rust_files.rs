use std::borrow::Cow;
use std::io;
use std::path::PathBuf;

use file_ports::{Files, Kind};

use crate::cli::report::failure::Failure;

#[derive(Debug)]
pub struct RustFiles;

#[derive(Debug)]
enum Entry {
    Rust(PathBuf),
    Directory,
}

type Listing = Result<Vec<PathBuf>, Failure<'static>>;

impl RustFiles {
    const RUST: &'static str = "rs";

    pub fn below(files: &impl Files, directory: PathBuf) -> Listing {
        Self::listed(files, vec![directory]).into_iter().next().unwrap_or_else(|| Ok(Vec::new()))
    }

    fn listed(files: &impl Files, level: Vec<PathBuf>) -> Vec<Listing> {
        let listings = files.entries_many(&level);
        let mut kinds = files.kinds(&listings.iter().flatten().flatten().collect::<Vec<&PathBuf>>()).into_iter();
        let mut deeper = Vec::new();
        let entries: Vec<Result<Vec<Entry>, Failure<'static>>> = level
            .into_iter()
            .zip(listings)
            .map(|(directory, listing)| {
                listing
                    .map(|paths| {
                        paths
                            .into_iter()
                            .zip(&mut kinds)
                            .filter_map(|(path, kind)| match kind {
                                Kind::Directory => {
                                    deeper.push(path);
                                    Some(Entry::Directory)
                                }
                                Kind::File | Kind::Link | Kind::Missing => Self::rust(path).map(Entry::Rust),
                            })
                            .collect()
                    })
                    .map_err(|failure| Failure::Io { path: Cow::Owned(directory), source: io::Error::from(failure) })
            })
            .collect();
        let mut below = if deeper.is_empty() { Vec::new() } else { Self::listed(files, deeper) }.into_iter();
        entries.into_iter().map(|listing| Self::flattened(listing?, &mut below)).collect()
    }

    fn flattened(entries: Vec<Entry>, below: &mut impl Iterator<Item = Listing>) -> Listing {
        let found: Vec<Listing> = entries
            .into_iter()
            .map(|entry| match entry {
                Entry::Rust(path) => Ok(vec![path]),
                Entry::Directory => below.next().unwrap_or_else(|| Ok(Vec::new())),
            })
            .collect();
        found.into_iter().try_fold(Vec::new(), |found, more| Ok(found.into_iter().chain(more?).collect()))
    }

    fn rust(file: PathBuf) -> Option<PathBuf> {
        Some(file).filter(|file| file.extension().is_some_and(|extension| extension == Self::RUST))
    }
}
