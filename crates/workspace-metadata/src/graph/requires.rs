use std::collections::{BTreeMap, BTreeSet};

use algebra::{Monoid, Semigroup};

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

impl Semigroup for Requires<'_> {
    fn combine(self, other: Self) -> Self {
        Requires(other.0.into_iter().fold(self.0, |mut all, (dependent, required)| {
            all.entry(dependent).or_default().extend(required);
            all
        }))
    }
}

impl Monoid for Requires<'_> {}

impl<'meta> Requires<'meta> {
    pub(super) fn linked(nodes: &'meta [Node], linkage: Linkage) -> Requires<'meta> {
        Monoid::concat(nodes.iter().map(|node| Requires(BTreeMap::from([(node.package_id(), linkage.required(node))]))))
    }

    pub(super) fn direct(&self, dependent: &PackageId) -> impl Iterator<Item = &'meta PackageId> + use<'_, 'meta> {
        self.0.get(dependent).into_iter().flatten().copied()
    }
}
