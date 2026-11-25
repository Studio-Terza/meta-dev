mod bypasses;
mod chain;
mod claims;
mod closure;
mod requires;
pub(crate) mod roles;
mod served;
mod stale;
mod unit;

use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use newtype::Wrapping;

use crate::crate_name::CrateName;
use crate::metadata::{Metadata, Package, PortTable};
use crate::package_id::PackageId;
use crate::role::Role;
pub use bypasses::{Bypass, Bypasses, DependencyKeys};
use chain::Chain;
pub use chain::Chained;
use claims::Claims;
use closure::Closure;
use requires::Requires;
use roles::{Expansion, RoleBreaches};
use served::Served;
pub use stale::Stale;
use unit::Unit;

#[derive(Debug, Default)]
pub struct Graph<'meta> {
    packages: BTreeMap<&'meta PackageId, &'meta Package>,
    named: BTreeMap<&'meta CrateName, BTreeSet<&'meta PackageId>>,
    members: BTreeSet<&'meta PackageId>,
    requires: Requires<'meta>,
    normal: Requires<'meta>,
}

impl<'meta> FromIterator<&'meta Metadata> for Graph<'meta> {
    fn from_iter<Workspaces: IntoIterator<Item = &'meta Metadata>>(workspaces: Workspaces) -> Graph<'meta> {
        workspaces.into_iter().fold(Graph::default(), |mut graph, metadata| {
            metadata.packages().iter().filter_map(|package| Some((package.package_id()?, package))).for_each(
                |(id, package)| {
                    graph.packages.insert(id, package);
                    graph.named.entry(AsRef::<CrateName>::as_ref(package)).or_default().insert(id);
                },
            );
            graph.members.extend(metadata.members());
            graph.requires.extend(metadata.nodes());
            graph.normal.joined(Requires::normal(metadata.nodes()));
            graph
        })
    }
}

impl<'meta> Graph<'meta> {
    #[must_use]
    pub fn chained(&self) -> Vec<Chained<'meta>> {
        let below = self.members.iter().map(|id| {
            let reached = Closure::reached(&self.normal, BTreeSet::from([*id])).into_inner();
            (*id, reached.intersection(&self.members).copied().filter(|other| other != id).collect())
        });
        Chain::from(below.collect::<BTreeMap<_, _>>())
            .placed(|id| self.packages.get(id).map(|package| AsRef::<CrateName>::as_ref(*package)))
    }

    #[must_use]
    pub fn bypasses(&self) -> Bypasses<'meta> {
        let units = self.units();
        let claims = Claims::held(self, &units);
        self.members
            .iter()
            .flat_map(|dependent| self.requires.direct(dependent).map(move |required| (*dependent, required)))
            .filter_map(|(dependent, required)| {
                let owners: BTreeSet<&CrateName> = claims
                    .owners(dependent, required)
                    .into_iter()
                    .filter_map(|owner| Some(AsRef::<CrateName>::as_ref(self.package(owner)?)))
                    .collect();
                let required_name = AsRef::<CrateName>::as_ref(self.package(required)?);
                let dependent_package = self.package(dependent)?;
                Some(owners).filter(|named| named.first().is_some()).map(|named| {
                    (
                        AsRef::<CrateName>::as_ref(dependent_package),
                        required_name,
                        Bypass::new(named, dependent_package.keys(required_name).collect()),
                    )
                })
            })
            .collect()
    }

    #[must_use]
    pub fn role_breaches(&self) -> RoleBreaches<'meta> {
        RoleBreaches::found(self, &self.units())
    }

    #[must_use]
    pub fn expansions(&self) -> Vec<Expansion<'meta>> {
        Expansion::found(self, &self.units())
    }

    #[must_use]
    pub fn serving(&self, role: Role) -> Vec<&'meta CrateName> {
        let ported = self.members.iter().copied().filter(|id| {
            self.package(id).and_then(Package::port).and_then(PortTable::dependents).is_some_and(|named| named == role)
        });
        let named: Served<'meta> = self.declaring(role).into_iter().chain(ported).collect();
        named.closed(self).iter().filter_map(|id| Some(AsRef::<CrateName>::as_ref(self.package(id)?))).collect()
    }

    pub(super) fn declaring(&self, role: Role) -> BTreeSet<&'meta PackageId> {
        self.members
            .iter()
            .copied()
            .filter(|id| self.package(id).is_some_and(|package| package.roles().contains(&role)))
            .collect()
    }

    pub(super) fn normal_reach(
        &self,
        start: &'meta PackageId,
        avoided: &BTreeSet<&'meta PackageId>,
    ) -> BTreeSet<&'meta PackageId> {
        Closure::avoiding(&self.normal, BTreeSet::from([start]), avoided).into_inner()
    }

    #[must_use]
    pub fn stale(&self) -> Stale<'meta> {
        self.units().iter().map(|unit| unit.unreached(self)).collect()
    }

    fn units(&self) -> Vec<Unit<'meta>> {
        let macros: BTreeSet<&PackageId> = self
            .members
            .iter()
            .copied()
            .filter(|id| self.package(id).and_then(Package::proc_macro).is_some())
            .collect();
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
                Some(Unit::new(port, members, declarations))
            })
            .collect()
    }

    fn reach(&self, unit: &Unit<'meta>) -> BTreeSet<&'meta PackageId> {
        Closure::reached(&self.requires, unit.members().clone()).into_inner()
    }

    fn resolved(&self, name: &CrateName) -> impl Iterator<Item = &'meta PackageId> + use<'_, 'meta> {
        self.named.get(name).into_iter().flatten().copied()
    }

    fn package(&self, id: &PackageId) -> Option<&'meta Package> {
        self.packages.get(id).copied()
    }
}
