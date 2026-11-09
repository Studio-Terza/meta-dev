use std::collections::{BTreeMap, BTreeSet};

use crate::crate_name::CrateName;
use crate::metadata::PortTable;
use crate::package_id::PackageId;

use super::Graph;
use super::stale::Stale;

#[derive(Debug)]
pub(super) struct Unit<'meta> {
    port: &'meta PackageId,
    members: BTreeSet<&'meta PackageId>,
    declarations: BTreeMap<&'meta PackageId, &'meta PortTable>,
}

impl<'meta> Unit<'meta> {
    pub(super) fn new(
        port: &'meta PackageId,
        members: BTreeSet<&'meta PackageId>,
        declarations: BTreeMap<&'meta PackageId, &'meta PortTable>,
    ) -> Unit<'meta> {
        Unit { port, members, declarations }
    }

    pub(super) fn port(&self) -> &'meta PackageId {
        self.port
    }

    pub(super) fn members(&self) -> &BTreeSet<&'meta PackageId> {
        &self.members
    }

    pub(super) fn declarations(&self) -> &BTreeMap<&'meta PackageId, &'meta PortTable> {
        &self.declarations
    }

    pub(super) fn unreached(&self, graph: &Graph<'meta>) -> Stale<'meta> {
        let reach = graph.reach(self);
        self.declarations
            .iter()
            .flat_map(|(declarer, declared)| declared.owns().iter().map(move |owned| (*declarer, owned)))
            .filter_map(|(declarer, owned)| {
                Some((graph.package(declarer)?.name(), Unit::outside(graph, &reach, owned)?))
            })
            .collect()
    }

    fn outside(
        graph: &Graph<'meta>,
        reach: &BTreeSet<&'meta PackageId>,
        owned: &'meta CrateName,
    ) -> Option<&'meta CrateName> {
        match graph.resolved(owned).find(|id| reach.contains(id)) {
            Some(_reached) => None,
            None => Some(owned),
        }
    }

    pub(super) fn holds(&self, package: &PackageId) -> Option<&'meta PackageId> {
        self.members.get(package).copied()
    }

    pub(super) fn shares(&self, package: &CrateName) -> Option<&'meta CrateName> {
        self.declarations.values().find_map(|declared| declared.shares(package))
    }
}
