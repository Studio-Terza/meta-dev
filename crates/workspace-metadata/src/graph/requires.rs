use std::collections::{BTreeMap, BTreeSet};

use crate::metadata::Node;
use crate::package_id::PackageId;

#[derive(Debug, Default)]
pub(super) struct Requires<'meta>(BTreeMap<&'meta PackageId, BTreeSet<&'meta PackageId>>);

impl<'meta> Extend<&'meta Node> for Requires<'meta> {
    fn extend<Nodes: IntoIterator<Item = &'meta Node>>(&mut self, nodes: Nodes) {
        nodes.into_iter().for_each(|node| self.0.entry(node.as_ref()).or_default().extend(node.dependencies()));
    }
}

impl<'meta> Requires<'meta> {
    pub(super) fn direct(&self, dependent: &PackageId) -> impl Iterator<Item = &'meta PackageId> + use<'_, 'meta> {
        self.0.get(dependent).into_iter().flatten().copied()
    }
}
