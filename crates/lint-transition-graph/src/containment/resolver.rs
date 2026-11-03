mod reach;
mod stand_ins;

use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use lint_core::{Site, SourceFile};
use lint_item_graph::{ItemGraph, NodeIndex, Scope, Traversal};
use lint_layout::Name;
use lint_placement::CrateName;
use transition_facts::{Endpoint, Nature, Transition};
use type_facts::Dependency;

use super::link::Link;
use super::member::Member;
use reach::Reach;
pub(crate) use stand_ins::StandIns;

#[derive(Debug, Default)]
pub(crate) struct Resolver<'found> {
    declaring: BTreeMap<&'found Name, Vec<Member<'found>>>,
    modules: BTreeMap<&'found CrateName, BTreeSet<&'found Name>>,
    scopes: BTreeMap<&'found Path, Scope<'found>>,
    stand_ins: StandIns<'found>,
}

impl<'found> Resolver<'found> {
    pub(crate) fn of_members(
        members: impl Iterator<Item = Member<'found>>,
        files: &[&'found SourceFile],
        stand_ins: StandIns<'found>,
    ) -> Resolver<'found> {
        let modules = files.iter().fold(BTreeMap::new(), |mut modules: BTreeMap<_, BTreeSet<_>>, file| {
            let placement = file.placement();
            modules.entry(placement.package().crate_name()).or_default().extend(placement.module().segments());
            modules
        });
        Resolver {
            declaring: members.fold(BTreeMap::new(), |mut declaring: BTreeMap<_, Vec<_>>, member| {
                declaring.entry(member.name()).or_default().push(member);
                declaring
            }),
            modules,
            scopes: files.iter().map(|file| (file.path(), Scope::of_file(file.syntax()))).collect(),
            stand_ins,
        }
    }

    pub(crate) fn members(&self, from: &CrateName, name: &Name, hint: Option<&Name>) -> Vec<Member<'found>> {
        let declaring = self.declaring.get(name).map_or(&[][..], Vec::as_slice);
        let single = || declaring.first().copied().filter(|_| declaring.len() == 1).into_iter().collect();
        let reached = Reach::of_hint(hint, from, &self.modules).krate(from);
        reached.map_or_else(Vec::new, |krate| {
            Some(self.within(krate, name)).filter(|found| !found.is_empty()).unwrap_or_else(single)
        })
    }

    fn in_crate(declaring: Option<&Vec<Member<'found>>>, krate: &CrateName) -> Option<Member<'found>> {
        declaring?.iter().find(|member| member.cluster().krate() == Some(krate)).copied()
    }

    fn within(&self, krate: &CrateName, name: &Name) -> Vec<Member<'found>> {
        let declared = |name: &Name| Self::in_crate(self.declaring.get(name), krate);
        let stood = |name: &Name| self.stand_ins.standing_for(krate, name).iter();
        declared(name).map_or_else(
            || Traversal::trees(stood(name), stood).into_iter().flatten().filter_map(declared).collect(),
            |member| vec![member],
        )
    }

    pub(crate) fn dependency(&self, from: &CrateName, dependency: &Dependency<'_>) -> Vec<Member<'found>> {
        let name: &Name = dependency.as_ref();
        let site: &Site<'_> = dependency.as_ref();
        let written = dependency.qualifier();
        let origin = self.scopes.get(site.path()).map_or_else(|| written.cloned(), |scope| scope.origin(name, written));
        self.members(from, name, origin.as_ref())
    }

    pub(crate) fn endpoint(&self, from: &CrateName, endpoint: &Endpoint) -> Vec<Member<'found>> {
        match endpoint.nature() {
            Nature::Named => self.members(from, endpoint.as_ref(), endpoint.origin().map(Borrow::borrow)),
            Nature::Interface => self.members(from, endpoint.as_ref(), None),
            Nature::Primitive | Nature::Boxed => Vec::new(),
        }
    }

    pub(crate) fn bodies(&self, krate: &CrateName, graph: &ItemGraph) -> Vec<Link<'found>> {
        let owners = graph.links().iter().enumerate();
        owners.flat_map(|(index, used)| self.body(krate, graph, NodeIndex::from(index), used)).collect()
    }

    fn body(&self, krate: &CrateName, graph: &ItemGraph, owner: NodeIndex, used: &[NodeIndex]) -> Vec<Link<'found>> {
        let local = |index: NodeIndex| {
            let name = graph.name_at(index).map_or_else(Vec::new, |name| self.members(krate, name, None));
            name.into_iter().filter(|member| member.cluster().krate() == Some(krate))
        };
        let holder = graph.name_at(owner).and_then(|name| Self::in_crate(self.declaring.get(name), krate));
        used.iter()
            .flat_map(|index| local(*index))
            .filter_map(|end| holder.map(|holder| Link::new(holder, end)))
            .collect()
    }

    pub(crate) fn signature(&self, transition: Transition<'found, '_>) -> Vec<Link<'found>> {
        let from: &CrateName = transition.context().as_ref();
        let holders = transition.context().subject().map_or_else(Vec::new, |subject| self.endpoint(from, subject));
        let ends = transition.takes().iter().chain([transition.gives().endpoint()]);
        ends.flat_map(|end| self.endpoint(from, end))
            .flat_map(|end| holders.iter().map(move |holder| Link::new(*holder, end)))
            .collect()
    }
}
