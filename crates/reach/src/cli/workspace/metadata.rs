mod package;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::Deserialize;

use crate::cli::failure::Failure;
use crate::cli::workspace::crate_name::CrateName;
use package::Package;

#[derive(Debug, Deserialize)]
pub struct Metadata {
    workspace_root: PathBuf,
    packages: Vec<Package>,
}

impl Metadata {
    pub fn directories(&self) -> Result<BTreeMap<CrateName, PathBuf>, Failure> {
        self.packages
            .iter()
            .map(|package| Ok((package.name().clone(), package.directory_under(&self.workspace_root)?)))
            .collect()
    }

    pub fn dependents(&self, members: &BTreeSet<CrateName>) -> BTreeMap<CrateName, BTreeSet<CrateName>> {
        self.packages
            .iter()
            .flat_map(|package| package.dependencies_among(members).map(move |dependency| (dependency, package.name())))
            .fold(BTreeMap::new(), |mut dependents, (dependency, name)| {
                dependents.entry(dependency.clone()).or_insert_with(BTreeSet::new).insert(name.clone());
                dependents
            })
    }
}
