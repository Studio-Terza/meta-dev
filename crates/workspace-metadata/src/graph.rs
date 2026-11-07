use std::collections::{BTreeMap, BTreeSet};

use crate::crate_name::CrateName;
use crate::metadata::{Metadata, Package};
use crate::package_id::PackageId;

#[derive(Debug, Default)]
pub struct Graph<'meta> {
    packages: BTreeMap<&'meta PackageId, &'meta Package>,
    members: BTreeSet<&'meta PackageId>,
    requires: BTreeMap<&'meta PackageId, BTreeSet<&'meta PackageId>>,
}

impl<'meta> FromIterator<&'meta Metadata> for Graph<'meta> {
    fn from_iter<Workspaces: IntoIterator<Item = &'meta Metadata>>(workspaces: Workspaces) -> Graph<'meta> {
        workspaces.into_iter().fold(Graph::default(), |mut graph, metadata| {
            graph
                .packages
                .extend(metadata.packages().iter().filter_map(|package| Some((package.identity()?, package))));
            graph.members.extend(metadata.members());
            metadata
                .nodes()
                .iter()
                .for_each(|node| graph.requires.entry(node.as_ref()).or_default().extend(node.dependencies()));
            graph
        })
    }
}

impl<'meta> Graph<'meta> {
    #[must_use]
    pub fn macros(&self) -> BTreeMap<&'meta CrateName, BTreeSet<&'meta CrateName>> {
        self.members
            .iter()
            .filter_map(|id| {
                let required = self
                    .requires
                    .get(id)
                    .into_iter()
                    .flatten()
                    .filter(|required| self.members.contains(*required))
                    .filter_map(|required| self.packages.get(required).copied().and_then(Package::proc_macro))
                    .map(Package::name)
                    .collect();
                Some((self.packages.get(id)?.name(), required))
            })
            .collect()
    }
}
