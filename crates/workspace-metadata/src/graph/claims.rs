mod claim;
mod named;
mod reached;

use std::collections::{BTreeMap, BTreeSet};

use newtype::Newtype;

use crate::package_id::PackageId;

use super::Graph;
use super::closure::Closure;
use super::unit::Unit;
use claim::Claim;
use named::Named;
use reached::Reached;

#[derive(Debug)]
pub(super) struct Claims<'unit, 'meta> {
    named: Named<'unit, 'meta>,
    reached: Reached<'unit, 'meta>,
}

impl<'unit, 'meta> Claims<'unit, 'meta> {
    pub(super) fn held(graph: &Graph<'meta>, units: &'unit [Unit<'meta>]) -> Claims<'unit, 'meta> {
        let named: Vec<Claim<'unit, 'meta>> = units.iter().flat_map(|unit| Self::named_in(graph, unit)).collect();
        let reached: Vec<Claim<'unit, 'meta>> =
            units.iter().flat_map(|unit| Self::reached_in(graph, unit, &named)).collect();
        Claims { named: Named::new(Self::grouped(named)), reached: Reached::new(Self::grouped(reached)) }
    }

    pub(super) fn owners(&self, dependent: &PackageId, required: &PackageId) -> BTreeSet<&'meta PackageId> {
        self.named.owners(dependent, required).unwrap_or_else(|| self.reached.owners(dependent, required))
    }

    fn named_in(graph: &Graph<'meta>, unit: &'unit Unit<'meta>) -> Vec<Claim<'unit, 'meta>> {
        unit.declarations()
            .iter()
            .flat_map(|(declarer, declared)| {
                declared
                    .owns()
                    .iter()
                    .flat_map(|owned| graph.resolved(owned))
                    .map(|id| Claim::named(id, declarer, unit))
            })
            .collect()
    }

    fn reached_in(
        graph: &Graph<'meta>,
        unit: &'unit Unit<'meta>,
        named: &[Claim<'unit, 'meta>],
    ) -> Vec<Claim<'unit, 'meta>> {
        let reach = graph.reach(unit);
        let owned = named
            .iter()
            .filter(|claim| claim.unit().port() == unit.port())
            .map(Claim::package)
            .filter(|id| reach.contains(id))
            .collect();
        Closure::reached(&graph.requires, owned)
            .into_inner()
            .into_iter()
            .filter_map(|id| Some(Claim::reached(id, graph.package(id)?, unit)))
            .collect()
    }

    fn grouped(found: Vec<Claim<'unit, 'meta>>) -> BTreeMap<&'meta PackageId, Vec<Claim<'unit, 'meta>>> {
        found.into_iter().fold(BTreeMap::new(), |mut claims, claim| {
            claims.entry(claim.package()).or_insert_with(Vec::new).push(claim);
            claims
        })
    }
}
