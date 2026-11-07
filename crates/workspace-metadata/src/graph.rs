mod claim;
mod claims;
mod closure;
mod requires;
mod unit;

use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use newtype::Newtype;

use crate::crate_name::CrateName;
use crate::metadata::{Metadata, Package};
use crate::package_id::PackageId;
use claim::Claim;
use claims::Claims;
use closure::Closure;
use requires::Requires;
use unit::Unit;

#[derive(Debug, Default)]
pub struct Graph<'meta> {
    packages: BTreeMap<&'meta PackageId, &'meta Package>,
    members: BTreeSet<&'meta PackageId>,
    requires: Requires<'meta>,
}

impl<'meta> FromIterator<&'meta Metadata> for Graph<'meta> {
    fn from_iter<Workspaces: IntoIterator<Item = &'meta Metadata>>(workspaces: Workspaces) -> Graph<'meta> {
        workspaces.into_iter().fold(Graph::default(), |mut graph, metadata| {
            graph
                .packages
                .extend(metadata.packages().iter().filter_map(|package| Some((package.identity()?, package))));
            graph.members.extend(metadata.members());
            graph.requires.extend(metadata.nodes());
            graph
        })
    }
}

impl<'meta> Graph<'meta> {
    #[must_use]
    pub fn bypasses(&self) -> BTreeMap<&'meta CrateName, BTreeMap<&'meta CrateName, &'meta CrateName>> {
        let units = self.units();
        let claims: Claims<'_, 'meta> = units.iter().flat_map(|unit| self.claims(unit)).collect();
        self.members
            .iter()
            .flat_map(|dependent| self.requires.direct(dependent).map(move |required| (*dependent, required)))
            .filter_map(|(dependent, required)| {
                let owner = claims.owner(dependent, required)?;
                Some((self.package(dependent)?.name(), self.package(required)?.name(), self.package(owner)?.name()))
            })
            .fold(BTreeMap::new(), |mut found, (dependent, required, owner)| {
                found.entry(dependent).or_insert_with(BTreeMap::new).insert(required, owner);
                found
            })
    }

    fn units(&self) -> Vec<Unit<'meta>> {
        let macros: BTreeSet<&PackageId> =
            self.members.iter().copied().filter(|id| self.package(id).and_then(Package::proc_macro).is_some()).collect();
        let ports: BTreeMap<&PackageId, BTreeSet<&PackageId>> = self
            .members
            .iter()
            .filter_map(|id| self.package(id)?.port().and(Some(*id)))
            .map(|port| (port, self.requires.direct(port).filter(|required| macros.contains(required)).collect()))
            .collect();
        let absorbed: BTreeSet<&PackageId> = ports.values().flatten().copied().collect();
        let declaring: BTreeSet<&PackageId> = ports.keys().copied().collect();
        declaring
            .difference(&absorbed)
            .filter_map(|port| {
                let members: BTreeSet<&PackageId> = iter::once(*port).chain(ports.get(port)?.iter().copied()).collect();
                let declarations =
                    members.iter().filter_map(|member| Some((*member, self.package(member)?.port()?))).collect();
                Some(Unit::new(members, declarations))
            })
            .collect()
    }

    fn claims<'unit>(&self, unit: &'unit Unit<'meta>) -> Vec<Claim<'unit, 'meta>> {
        unit.declarations()
            .iter()
            .flat_map(|(owner, declared)| {
                let owned = Closure::reached(&self.requires, iter::once(*owner).collect())
                    .into_inner()
                    .into_iter()
                    .filter_map(|id| declared.owns(self.package(id)?).and(Some(id)))
                    .collect();
                Closure::reached(&self.requires, owned)
                    .into_inner()
                    .into_iter()
                    .filter_map(|id| Some(Claim::new(id, self.package(id)?, owner, unit)))
            })
            .collect()
    }

    fn package(&self, id: &PackageId) -> Option<&'meta Package> {
        self.packages.get(id).copied()
    }
}
