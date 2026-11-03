mod crate_count;
mod crate_name;
mod fan_in;
mod history;
mod hubs;
mod metadata;
mod percent;
mod reached;
mod readers;
mod share;
mod sources;

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use lint_complexity::Complexity;
use lint_core::SourceFile;

use crate_count::CrateCount;
use crate_name::CrateName;
use fan_in::FanIn;
use history::History;
use hubs::{Hub, Hubs};
pub use metadata::Metadata;
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
        (owners.is_empty() || reached.covering(&owners).is_none()).then_some(produced)
    }

    pub fn history(&self, changed: &[PathBuf]) -> History<'meta> {
        History::new(self.reached(changed), self.fan_in())
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
                .filter(|(name, _)| !self.closure(BTreeSet::from([**name])).is_disjoint(&wanted))
                .map(|(_, directory)| *directory)
                .collect(),
        )
    }

    pub fn fan_in(&self) -> FanIn<'meta> {
        self.directories.keys().fold(FanIn::among(CrateCount::new(self.directories.len())), |fan_in, name| {
            let reach = self.closure(BTreeSet::from([*name]));
            fan_in.counted(CrateCount::new(reach.iter().filter(|reached| *reached != name).count()), name)
        })
    }

    pub fn hubs<'file>(&self, sources: &'file [SourceFile], repository: &Path) -> Hubs<'meta, 'file> {
        let fan_in = self.fan_in();
        Hubs::weighed(
            Complexity::held(sources)
                .into_iter()
                .filter_map(|held| {
                    let owner = self.owner_of(repository, held.heaviest().path())?;
                    fan_in.share_of(owner).map(|share| Hub::new(owner, held, share))
                })
                .collect(),
        )
    }

    pub fn owner_of(&self, repository: &Path, path: &Path) -> Option<&'meta CrateName> {
        self.directories
            .iter()
            .filter(|(_, directory)| path.strip_prefix(repository).unwrap_or(path).starts_with(directory))
            .max_by_key(|(_, directory)| directory.components().count())
            .map(|(name, _)| *name)
    }

    fn touched(&self, path: &Path) -> BTreeSet<&'meta CrateName> {
        if path.file_name().is_some_and(|name| name == Self::MANIFEST) {
            self.everything()
        } else {
            self.owned(path).into_iter().chain(self.readers.reached_by(path)).collect()
        }
    }

    fn everything(&self) -> BTreeSet<&'meta CrateName> {
        self.directories.keys().copied().collect()
    }

    fn owned(&self, path: &Path) -> Vec<&'meta CrateName> {
        let owners: Vec<&CrateName> = self
            .directories
            .iter()
            .filter(|(_, directory)| !directory.as_os_str().is_empty() && path.starts_with(directory))
            .map(|(name, _)| *name)
            .collect();
        if owners.is_empty() && !Self::INERT.iter().any(|inert| path.starts_with(inert)) {
            self.everything().into_iter().collect()
        } else {
            owners
        }
    }

    fn closure(&self, crates: BTreeSet<&'meta CrateName>) -> BTreeSet<&'meta CrateName> {
        let grown: BTreeSet<&CrateName> =
            crates.iter().filter_map(|name| self.dependents.get(name)).flatten().chain(&crates).copied().collect();
        if grown.len() == crates.len() { crates } else { self.closure(grown) }
    }
}
