use std::collections::{BTreeMap, BTreeSet};

use crate::graph::DependencyKeys;
use crate::metadata::{Package, PortTable};
use crate::package_id::PackageId;

use super::super::Graph;
use super::super::unit::Unit;
use super::breaching::Breaching;
use super::unported::Unported;

#[derive(Debug)]
pub(super) struct Owned<'unit, 'meta>(BTreeMap<&'meta PackageId, Owner<'unit, 'meta>>);

#[derive(Clone, Copy, Debug)]
pub(super) struct Owner<'unit, 'meta> {
    package: &'meta PackageId,
    unit: &'unit Unit<'meta>,
    declared: &'meta PortTable,
}

impl<'unit, 'meta> Owned<'unit, 'meta> {
    pub(super) fn held(graph: &Graph<'meta>, units: &'unit [Unit<'meta>]) -> Owned<'unit, 'meta> {
        Owned(
            units
                .iter()
                .flat_map(|unit| unit.declarations().values().map(move |declared| (unit, *declared)))
                .flat_map(|(unit, declared)| {
                    declared
                        .owns()
                        .iter()
                        .flat_map(|name| graph.resolved(name))
                        .map(move |package| (package, Owner { package, unit, declared }))
                })
                .collect(),
        )
    }

    pub(super) fn first_in(&self, reach: &BTreeSet<&'meta PackageId>) -> Option<&'meta PackageId> {
        reach.iter().copied().find(|id| self.0.contains_key(id))
    }

    pub(super) fn within(&self, reach: &BTreeSet<&'meta PackageId>) -> Vec<Owner<'unit, 'meta>> {
        self.0.values().filter(|owner| reach.contains(owner.package)).copied().collect()
    }
}

impl<'meta> Owner<'_, 'meta> {
    pub(super) fn reached_by(&self, graph: &Graph<'meta>, package: &'meta Package) -> Option<Unported<'meta>> {
        let id = package.package_id().filter(|id| self.unit.holds(id).is_none())?;
        let role = self.declared.dependents();
        let avoided = role.map_or_default(|declared| graph.declaring(declared));
        let direct: BTreeSet<&PackageId> = graph.normal.direct(id).collect();
        let keys: DependencyKeys<'meta> = direct
            .difference(&avoided)
            .copied()
            .filter(|required| graph.normal_reach(required, &avoided).contains(self.package))
            .filter_map(|required| Some(graph.package(required)?.name()))
            .flat_map(|name| package.keys(name))
            .collect();
        let owned = graph.package(self.package)?.name();
        Some(keys)
            .filter(|found| found.as_ref().first().is_some())
            .map(|found| Unported::new(Breaching::new(package.name(), found), owned, role))
    }
}
