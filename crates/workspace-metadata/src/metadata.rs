mod library;
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
pub use library::Library;
pub use package::Form;
pub(crate) use package::{Package, PortTable};
pub(crate) use resolve::Node;
use resolve::{NodeDependency, Resolve};

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
    pub fn libraries(&self) -> Vec<Library<'_>> {
        let members: BTreeSet<&PackageId> = self.workspace_members.iter().collect();
        let named: BTreeMap<&PackageId, &CrateName> =
            self.packages.iter().filter_map(|package| Some((package.package_id()?, package.name()))).collect();
        let nodes: BTreeMap<&PackageId, &Node> = self.nodes().iter().map(|node| (node.package_id(), node)).collect();
        let direct: BTreeSet<&PackageId> = members
            .iter()
            .filter_map(|member| nodes.get(member))
            .flat_map(|node| node.normal().map(NodeDependency::package_id))
            .collect();
        let (near, far): (Vec<&Package>, Vec<&Package>) = self
            .packages
            .iter()
            .filter(|package| package.package_id().and_then(|id| members.get(id)).is_none())
            .partition(|package| package.package_id().and_then(|id| direct.get(id)).is_some());
        near.into_iter()
            .chain(far)
            .filter_map(|package| {
                let dependencies = package
                    .package_id()
                    .and_then(|id| nodes.get(id))
                    .into_iter()
                    .flat_map(|node| node.normal())
                    .filter_map(|dependency| Some((dependency.krate(), *named.get(dependency.package_id())?)))
                    .collect();
                let library = package.library()?;
                Some(Library::new(package.name(), library.root(), library.form()?, dependencies))
            })
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
        self.resolve.as_ref().map_or(&[], AsRef::as_ref)
    }
}

impl FromStr for Metadata {
    type Err = Unreadable;

    fn from_str(text: &str) -> Result<Metadata, Unreadable> {
        Ok(serde_json::from_str(text)?)
    }
}
