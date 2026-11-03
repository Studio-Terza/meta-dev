mod package;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::cli::failure::Failure;
use crate::cli::request::workspace::crate_name::CrateName;
use package::Package;

#[derive(Debug, Deserialize)]
pub struct Metadata {
    workspace_root: PathBuf,
    packages: Vec<Package>,
}

impl Metadata {
    pub fn checked(self) -> Result<Metadata, Failure> {
        let Metadata { workspace_root, packages } = self;
        let (inside, outside): (Vec<Package>, Vec<Package>) =
            packages.into_iter().partition(|package| package.directory_under(&workspace_root).is_some());
        outside.into_iter().next().map_or(Ok(Metadata { workspace_root, packages: inside }), |package| {
            Err(Failure::Outside { manifest: package.into_manifest() })
        })
    }

    pub fn directories(&self) -> BTreeMap<&CrateName, &Path> {
        self.packages
            .iter()
            .filter_map(|package| Some((package.name(), package.directory_under(&self.workspace_root)?)))
            .collect()
    }

    pub fn readers(&self) -> BTreeMap<&CrateName, &[PathBuf]> {
        self.packages
            .iter()
            .filter_map(|package| package.readings().first().map(|_| (package.name(), package.readings())))
            .collect()
    }

    pub fn dependents<'meta>(
        &'meta self,
        members: &BTreeMap<&CrateName, &Path>,
    ) -> BTreeMap<&'meta CrateName, BTreeSet<&'meta CrateName>> {
        self.packages
            .iter()
            .flat_map(|package| package.dependencies_among(members).map(move |dependency| (dependency, package.name())))
            .fold(BTreeMap::new(), |mut dependents, (dependency, name)| {
                dependents.entry(dependency).or_insert_with(BTreeSet::new).insert(name);
                dependents
            })
    }
}
