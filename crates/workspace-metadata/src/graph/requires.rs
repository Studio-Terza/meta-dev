use std::collections::{BTreeMap, BTreeSet};

use crate::metadata::{Node, NodeDependency};
use crate::package_id::PackageId;

#[derive(Debug, Default)]
pub(super) struct Requires<'meta>(BTreeMap<&'meta PackageId, BTreeSet<&'meta PackageId>>);

impl<'meta> Extend<&'meta Node> for Requires<'meta> {
    fn extend<Nodes: IntoIterator<Item = &'meta Node>>(&mut self, nodes: Nodes) {
        nodes.into_iter().for_each(|node| self.0.entry(node.package_id()).or_default().extend(node.dependencies()));
    }
}

impl<'meta> Requires<'meta> {
    pub(super) fn normal(nodes: &'meta [Node]) -> Requires<'meta> {
        Requires(
            nodes
                .iter()
                .map(|node| (node.package_id(), node.normal().map(NodeDependency::package_id).collect()))
                .collect(),
        )
    }

    pub(super) fn joined(&mut self, other: Requires<'meta>) {
        other.0.into_iter().for_each(|(dependent, required)| self.0.entry(dependent).or_default().extend(required));
    }

    pub(super) fn direct(&self, dependent: &PackageId) -> impl Iterator<Item = &'meta PackageId> + use<'_, 'meta> {
        self.0.get(dependent).into_iter().flatten().copied()
    }
}
