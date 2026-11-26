mod library;
mod package;
mod resolve;

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use crate::crate_name::CrateName;
use crate::package_id::PackageId;
pub use library::Library;
pub use package::Form;
use package::Package;
use resolve::{Node, NodeDependency, Resolve};

#[derive(Clone, Debug, Deserialize)]
pub struct Metadata {
    packages: Vec<Package>,
    #[serde(default)]
    workspace_members: Vec<PackageId>,
    resolve: Option<Resolve>,
}

impl Metadata {
    #[must_use]
    pub fn libraries(&self) -> Vec<Library<'_>> {
        let members: BTreeSet<&PackageId> = self.workspace_members.iter().collect();
        let named: BTreeMap<&PackageId, &CrateName> = self
            .packages
            .iter()
            .filter_map(|package| Some((package.package_id()?, AsRef::<CrateName>::as_ref(package))))
            .collect();
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
                Some(Library::new(AsRef::<CrateName>::as_ref(package), library.root(), library.form()?, dependencies))
            })
            .collect()
    }

    fn nodes(&self) -> &[Node] {
        self.resolve.as_ref().map_or(&[], AsRef::as_ref)
    }
}
