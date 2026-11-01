mod crate_count;
mod crate_name;
mod fan_in;
mod history;
mod metadata;
mod reached;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::cli::failure::Failure;
use crate_count::CrateCount;
use crate_name::CrateName;
use fan_in::FanIn;
use history::History;
use metadata::Metadata;
use reached::Reached;

#[derive(Debug)]
pub struct Workspace {
    directories: BTreeMap<CrateName, PathBuf>,
    dependents: BTreeMap<CrateName, BTreeSet<CrateName>>,
}

impl FromStr for Workspace {
    type Err = Failure;

    fn from_str(metadata: &str) -> Result<Workspace, Failure> {
        let metadata: Metadata = serde_json::from_str(metadata)?;
        let directories = metadata.directories()?;
        let dependents = metadata.dependents(&directories.keys().cloned().collect());
        Ok(Workspace { directories, dependents })
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

    pub fn history(&self, changed: &[PathBuf]) -> History {
        History::new(self.reached(changed), self.fan_in())
    }

    pub fn fan_in(&self) -> FanIn {
        self.directories.keys().fold(FanIn::default(), |fan_in, name| {
            let reach = self.closure(BTreeSet::from([name.clone()]));
            fan_in.counted(CrateCount::new(reach.iter().filter(|reached| *reached != name).count()), name.clone())
        })
    }

    fn touched(&self, path: &Path) -> BTreeSet<CrateName> {
        if path.file_name().is_some_and(|name| name == Self::MANIFEST) {
            self.everything()
        } else {
            self.owned(path).into_iter().collect()
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
