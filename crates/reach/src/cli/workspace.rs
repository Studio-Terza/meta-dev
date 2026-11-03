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
use std::str::FromStr;

use file_ports::Files;
use lint_complexity::Complexity;
use lint_reader::SourceFiles;

use crate::cli::failure::Failure;
use crate_count::CrateCount;
use crate_name::CrateName;
use fan_in::FanIn;
use history::History;
use hubs::{Hub, Hubs};
use metadata::Metadata;
use reached::Reached;
use readers::Readers;
use sources::Sources;

#[derive(Debug)]
pub struct Workspace {
    directories: BTreeMap<CrateName, PathBuf>,
    dependents: BTreeMap<CrateName, BTreeSet<CrateName>>,
    readers: Readers,
}

impl FromStr for Workspace {
    type Err = Failure;

    fn from_str(metadata: &str) -> Result<Workspace, Failure> {
        let metadata: Metadata = serde_json::from_str(metadata)?;
        let directories = metadata.directories()?;
        let dependents = metadata.dependents(&directories.keys().cloned().collect());
        Ok(Workspace { directories, dependents, readers: Readers::new(metadata.readers()) })
    }
}

impl Workspace {
    const INERT: [&'static str; 2] = ["README.md", "specs"];
    const MANIFEST: &'static str = "Cargo.toml";

    pub fn reached(&self, changed: &[PathBuf]) -> Reached {
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

    pub fn history(&self, changed: &[PathBuf]) -> History {
        History::new(self.reached(changed), self.fan_in())
    }

    pub fn sources(&self, roots: &[OsString]) -> Sources {
        let wanted: BTreeSet<CrateName> = self
            .directories
            .keys()
            .filter(|name| roots.iter().any(|root| root.to_str() == Some(name.to_string().as_str())))
            .cloned()
            .collect();
        Sources::new(
            self.directories
                .iter()
                .filter(|(name, _)| !self.closure(BTreeSet::from([(*name).clone()])).is_disjoint(&wanted))
                .map(|(_, directory)| directory.clone())
                .collect(),
        )
    }

    pub fn fan_in(&self) -> FanIn {
        self.directories.keys().fold(FanIn::among(CrateCount::new(self.directories.len())), |fan_in, name| {
            let reach = self.closure(BTreeSet::from([name.clone()]));
            fan_in.counted(CrateCount::new(reach.iter().filter(|reached| *reached != name).count()), name.clone())
        })
    }

    pub fn hubs(&self, files: &impl Files, repository: &Path) -> Result<Hubs, Failure> {
        let readout = SourceFiles::read(files, repository)?;
        let fan_in = self.fan_in();
        Ok(Hubs::weighed(
            Complexity::held(readout.files())
                .into_iter()
                .filter_map(|held| {
                    let owner = self.owner_of(repository, held.heaviest().path())?;
                    fan_in.share_of(owner).map(|share| Hub::new(owner.clone(), held, share))
                })
                .collect(),
        ))
    }

    pub fn owner_of(&self, repository: &Path, path: &Path) -> Option<&CrateName> {
        self.directories
            .iter()
            .filter(|(_, directory)| path.strip_prefix(repository).unwrap_or(path).starts_with(directory))
            .max_by_key(|(_, directory)| directory.components().count())
            .map(|(name, _)| name)
    }

    fn touched(&self, path: &Path) -> BTreeSet<CrateName> {
        if path.file_name().is_some_and(|name| name == Self::MANIFEST) {
            self.everything()
        } else {
            self.owned(path).into_iter().chain(self.readers.reached_by(path)).collect()
        }
    }

    fn everything(&self) -> BTreeSet<CrateName> {
        self.directories.keys().cloned().collect()
    }

    fn owned(&self, path: &Path) -> Vec<CrateName> {
        let owners: Vec<CrateName> = self
            .directories
            .iter()
            .filter(|(_, directory)| !directory.as_os_str().is_empty() && path.starts_with(directory))
            .map(|(name, _)| name.clone())
            .collect();
        if owners.is_empty() && !Self::INERT.iter().any(|inert| path.starts_with(inert)) {
            self.everything().into_iter().collect()
        } else {
            owners
        }
    }

    fn closure(&self, crates: BTreeSet<CrateName>) -> BTreeSet<CrateName> {
        let grown: BTreeSet<CrateName> =
            crates.iter().filter_map(|name| self.dependents.get(name)).flatten().chain(&crates).cloned().collect();
        if grown.len() == crates.len() { crates } else { self.closure(grown) }
    }
}
