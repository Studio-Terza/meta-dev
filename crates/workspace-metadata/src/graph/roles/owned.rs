use crate::crate_name::CrateName;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Not;

use crate::graph::DependencyKeys;
use crate::metadata::{Package, PortTable};
use crate::package_id::PackageId;

use super::super::Graph;
use super::super::unit::Unit;
use super::breaching::Breaching;
use super::unported::Unported;

#[derive(Debug)]
pub(super) struct Owned<'unit, 'meta> {
    owners: BTreeMap<&'meta PackageId, Owner<'unit, 'meta>>,
    compiled: BTreeSet<&'meta PackageId>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Owner<'unit, 'meta> {
    package: &'meta PackageId,
    unit: &'unit Unit<'meta>,
    declared: &'meta PortTable,
}

impl<'unit, 'meta> Owned<'unit, 'meta> {
    pub(super) fn held(graph: &Graph<'meta>, units: &'unit [Unit<'meta>]) -> Owned<'unit, 'meta> {
        let owners = units
            .iter()
            .flat_map(|unit| unit.declarations().values().map(move |declared| (unit, *declared)))
            .flat_map(|(unit, declared)| {
                declared
                    .owns()
                    .iter()
                    .flat_map(|name| graph.resolved(name))
                    .map(move |package| (package, Owner { package, unit, declared }))
            })
            .collect();
        let compiled =
            graph.packages.iter().filter(|(_, package)| package.proc_macro().is_some()).map(|(id, _)| *id).collect();
        Owned { owners, compiled }
    }

    pub(super) fn compiled(&self) -> &BTreeSet<&'meta PackageId> {
        &self.compiled
    }

    pub(super) fn first_in(&self, reach: &BTreeSet<&'meta PackageId>) -> Option<&'meta PackageId> {
        reach.iter().copied().find(|id| self.owners.contains_key(id))
    }

    pub(super) fn within(&self, reach: &BTreeSet<&'meta PackageId>) -> Vec<Owner<'unit, 'meta>> {
        self.owners.values().filter(|owner| reach.contains(owner.package)).copied().collect()
    }
}

impl<'meta> Owner<'_, 'meta> {
    pub(super) fn reached_by(
        &self,
        graph: &Graph<'meta>,
        owned: &Owned<'_, 'meta>,
        package: &'meta Package,
    ) -> Option<Unported<'meta>> {
        let id = package.package_id().filter(|id| self.unit.members().contains(*id).not())?;
        let role = self.declared.dependents();
        let declaring = role.map_or_default(|declared| graph.declaring(declared));
        let avoided: BTreeSet<&PackageId> = declaring.union(owned.compiled()).copied().collect();
        let skipped: BTreeSet<&PackageId> = avoided.union(self.unit.members()).copied().collect();
        let direct: BTreeSet<&PackageId> = graph.normal.direct(id).collect();
        let keys: DependencyKeys<'meta> = direct
            .difference(&skipped)
            .copied()
            .filter(|required| graph.normal_reach(required, &avoided).contains(self.package))
            .filter_map(|required| Some(AsRef::<CrateName>::as_ref(graph.package(required)?)))
            .flat_map(|name| package.keys(name))
            .collect();
        let held = AsRef::<CrateName>::as_ref(graph.package(self.package)?);
        Some(keys)
            .filter(|found| found.as_ref().first().is_some())
            .map(|found| Unported::new(Breaching::new(AsRef::<CrateName>::as_ref(package), found), held, role))
    }
}
