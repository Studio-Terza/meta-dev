mod crate_count;
mod history;
mod reached;
mod readers;
mod sources;

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use workspace_metadata::{CrateName, Metadata};

use crate_count::CrateCount;
use history::History;
use reached::Reached;
use readers::Readers;
use sources::Sources;

#[derive(Debug)]
pub struct Workspace<'meta> {
    directories: BTreeMap<&'meta CrateName, &'meta Path>,
    dependents: BTreeMap<&'meta CrateName, BTreeSet<&'meta CrateName>>,
    readers: Readers<'meta>,
}

impl<'meta> From<&'meta Metadata> for Workspace<'meta> {
    fn from(metadata: &'meta Metadata) -> Workspace<'meta> {
        let directories = metadata.directories();
        let dependents = metadata.dependents(&directories);
        Workspace { directories, dependents, readers: Readers::new(metadata.readers()) }
    }
}

impl<'meta> Workspace<'meta> {
    const INERT: [&'static str; 2] = ["README.md", "specs"];
    const MANIFEST: &'static str = "Cargo.toml";

    pub fn reached(&self, changed: &[PathBuf]) -> Reached<'meta> {
        Reached::new(
            self.closure(changed.iter().flat_map(|path| self.touched(path)).collect()),
            CrateCount::new(self.directories.len()),
        )
    }

    pub fn beyond<'produced>(&self, produced: &'produced Path, changed: &[PathBuf]) -> Option<&'produced Path> {
        let owners = self.touched(produced);
        let reached = self.reached(changed);
        owners.first().and_then(|_| reached.covering(&owners)).map_or(Some(produced), |_| None)
    }

    pub fn history(&self, changed: &[PathBuf]) -> History<'meta> {
        History::new(self.reached(changed))
    }

    pub fn sources(&self, roots: &[OsString]) -> Sources<'meta> {
        let wanted: BTreeSet<&CrateName> = self
            .directories
            .keys()
            .filter(|name| roots.iter().any(|root| root.to_str() == Some(AsRef::<str>::as_ref(**name))))
            .copied()
            .collect();
        Sources::new(
            self.directories
                .iter()
                .filter(|(name, _)| {
                    self.closure(BTreeSet::from([**name])).iter().any(|reached| wanted.contains(reached))
                })
                .map(|(_, directory)| *directory)
                .collect(),
        )
    }

    fn touched(&self, path: &Path) -> BTreeSet<&'meta CrateName> {
        path.file_name().filter(|name| *name == Self::MANIFEST).map_or_else(
            || self.owned(path).into_iter().chain(self.readers.reached_by(path)).collect(),
            |_| self.everything(),
        )
    }

    fn everything(&self) -> BTreeSet<&'meta CrateName> {
        self.directories.keys().copied().collect()
    }

    fn owned(&self, path: &Path) -> Vec<&'meta CrateName> {
        let owners: Vec<&CrateName> = self
            .directories
            .iter()
            .filter(|(_, directory)| directory.components().next().is_some())
            .filter(|(_, directory)| path.starts_with(directory))
            .map(|(name, _)| *name)
            .collect();
        let inert = Self::INERT.iter().find(|inert| path.starts_with(inert));
        owners
            .first()
            .map(|_| ())
            .or(inert.map(|_| ()))
            .map_or_else(|| self.everything().into_iter().collect(), |()| owners)
    }

    fn closure(&self, crates: BTreeSet<&'meta CrateName>) -> BTreeSet<&'meta CrateName> {
        let grown: BTreeSet<&CrateName> =
            crates.iter().filter_map(|name| self.dependents.get(name)).flatten().chain(&crates).copied().collect();
        Some(grown).filter(|grown| grown.len() > crates.len()).map_or(crates, |grown| self.closure(grown))
    }
}
