mod package;
mod resolve;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Deserialize;

use crate::crate_name::CrateName;
use crate::outside::Outside;
use crate::package_id::PackageId;
use crate::readers::Readers;
use crate::unreadable::Unreadable;
pub(crate) use package::{Package, PortTable};
pub(crate) use resolve::Node;
use resolve::Resolve;

#[derive(Clone, Debug, Deserialize)]
pub struct Metadata {
    workspace_root: PathBuf,
    packages: Vec<Package>,
    #[serde(default)]
    workspace_members: Vec<PackageId>,
    resolve: Option<Resolve>,
}

impl Metadata {
    pub fn checked(self) -> Result<Metadata, Outside> {
        let Metadata { workspace_root, packages, workspace_members, resolve } = self;
        let (inside, outside): (Vec<Package>, Vec<Package>) =
            packages.into_iter().partition(|package| package.directory_under(&workspace_root).is_some());
        outside
            .into_iter()
            .next()
            .map_or(Ok(Metadata { workspace_root, packages: inside, workspace_members, resolve }), |package| {
                Err(Outside::new(package.into_manifest()))
            })
    }

    #[must_use]
    pub fn directories(&self) -> BTreeMap<&CrateName, &Path> {
        self.packages
            .iter()
            .filter_map(|package| Some((package.name(), package.directory_under(&self.workspace_root)?)))
            .collect()
    }

    #[must_use]
    pub fn readers(&self) -> Readers<'_> {
        Readers::new(
            self.packages
                .iter()
                .filter_map(|package| package.readings().first().map(|_| (package.name(), package.readings())))
                .collect(),
        )
    }

    #[must_use]
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

    pub(crate) fn packages(&self) -> &[Package] {
        &self.packages
    }

    pub(crate) fn members(&self) -> &[PackageId] {
        &self.workspace_members
    }

    pub(crate) fn nodes(&self) -> &[Node] {
        self.resolve.as_ref().map_or(&[], Resolve::nodes)
    }
}

impl FromStr for Metadata {
    type Err = Unreadable;

    fn from_str(text: &str) -> Result<Metadata, Unreadable> {
        Ok(serde_json::from_str(text)?)
    }
}
