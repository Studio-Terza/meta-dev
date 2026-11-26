use std::collections::{BTreeMap, BTreeSet};

use crate::metadata::{Node, NodeDependency};
use crate::package_id::PackageId;

#[derive(Debug, Default)]
pub(super) struct Requires<'meta>(BTreeMap<&'meta PackageId, BTreeSet<&'meta PackageId>>);

#[derive(Clone, Copy, Debug)]
pub(super) enum Linkage {
    Every,
    Normal,
}

impl Linkage {
    fn required(self, node: &Node) -> BTreeSet<&PackageId> {
        match self {
            Linkage::Every => node.dependencies().iter().collect(),
            Linkage::Normal => node.normal().map(NodeDependency::package_id).collect(),
        }
    }
}

impl<'meta> Requires<'meta> {
    pub(super) fn linked(nodes: &'meta [Node], linkage: Linkage) -> Requires<'meta> {
        Requires(nodes.iter().fold(BTreeMap::new(), |mut all: BTreeMap<_, BTreeSet<_>>, node| {
            all.entry(node.package_id()).or_default().extend(linkage.required(node));
            all
        }))
    }

    pub(super) fn direct(&self, dependent: &PackageId) -> impl Iterator<Item = &'meta PackageId> + use<'_, 'meta> {
        self.0.get(dependent).into_iter().flatten().copied()
    }
}
