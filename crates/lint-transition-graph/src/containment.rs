mod ask;
mod cluster;
mod connected;
mod groups;
mod interface;
mod layered;
mod link;
mod member;
mod members;
mod outsider;
mod placed;
mod slice;

use std::collections::{BTreeMap, BTreeSet};

use graph::Model;
use lint_core::{Headline, Report, Site, Violation};
use lint_layout::Name;

use cluster::Cluster;
use link::Link;
use member::Member;
use members::{Implemented, Members};
use outsider::Outsider;
use slice::Slice;

pub use ask::{Answer, Ask, Listed};
pub use connected::ConnectedClusters;
pub use interface::InterfaceByConsumers;
pub use layered::AcyclicQuotients;
pub use placed::LowestCluster;

pub trait ClusterLint {
    const HEADLINE: Headline;

    fn violations<'file>(view: &Containment<'_, 'file>) -> Vec<Violation<'file>>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Toward {
    Used,
    Users,
    Defined,
    Implementers,
}

#[derive(Debug)]
pub struct Containment<'found, 'file> {
    sites: BTreeMap<Member<'found>, Site<'file>>,
    uses: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
    users: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
    defines: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
    implementers: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
    outsiders: BTreeMap<Member<'found>, BTreeSet<Outsider<'found, 'file>>>,
    slices: BTreeMap<Cluster<'found>, Slice<'found>>,
}

impl<'found, 'file> Containment<'found, 'file> {
    #[must_use]
    pub fn clustered(model: &'file Model) -> Containment<'found, 'file>
    where
        'file: 'found,
    {
        let members = Members::from(model);
        let located = members.implemented();
        let implemented: BTreeSet<Link<'found>> =
            located.iter().filter_map(Implemented::link).filter_map(Link::apart).collect();
        let defined: BTreeSet<Link<'found>> =
            members.held().into_iter().filter_map(Link::apart).chain(implemented.iter().copied()).collect();
        let outsiders = located.iter().filter_map(Implemented::outsider).fold(
            BTreeMap::new(),
            |mut outsiders: BTreeMap<Member<'found>, BTreeSet<Outsider<'found, 'file>>>, outsider| {
                outsiders.entry(outsider.holder()).or_default().insert(outsider);
                outsiders
            },
        );
        let links: BTreeSet<Link<'found>> =
            defined.iter().copied().chain(members.used()).filter_map(Link::apart).collect();
        let users = Self::adjacency(links.iter().copied().map(Link::reversed));
        Containment {
            slices: Slice::all(members.declared(), &users),
            sites: members.sites().iter().map(|(member, site)| (*member, *site)).collect(),
            uses: Self::adjacency(links.iter().copied()),
            users,
            defines: Self::adjacency(defined.into_iter()),
            implementers: Self::adjacency(implemented.into_iter().map(Link::reversed)),
            outsiders,
        }
    }

    #[must_use]
    pub fn report<Check: ClusterLint>(&self) -> Report<'file> {
        Report::new(Check::HEADLINE, Check::violations(self))
    }

    fn toward(&self, toward: Toward) -> &BTreeMap<Member<'found>, BTreeSet<Member<'found>>> {
        match toward {
            Toward::Used => &self.uses,
            Toward::Users => &self.users,
            Toward::Defined => &self.defines,
            Toward::Implementers => &self.implementers,
        }
    }

    pub(crate) fn named<'view>(&'view self, name: &'view Name) -> impl Iterator<Item = Member<'found>> + 'view {
        self.all().filter(move |member| member.name() == name)
    }

    pub(crate) fn users_within(&self, member: Member<'found>) -> Option<Cluster<'found>> {
        self.adjacent(member, Toward::Users).map(|user| user.cluster()).reduce(Cluster::common)
    }

    pub(crate) fn outside(
        &self,
        member: Member<'found>,
        toward: Toward,
    ) -> impl Iterator<Item = Outsider<'found, 'file>> + '_ {
        Some(toward)
            .filter(|candidate| *candidate == Toward::Implementers)
            .into_iter()
            .flat_map(move |_| self.outsiders.get(&member).into_iter().flatten().cloned())
    }

    pub(crate) fn site(&self, member: Member<'found>) -> Option<Site<'file>> {
        self.sites.get(&member).copied()
    }

    pub(crate) fn adjacency(
        links: impl Iterator<Item = Link<'found>>,
    ) -> BTreeMap<Member<'found>, BTreeSet<Member<'found>>> {
        links.fold(BTreeMap::new(), |mut adjacency: BTreeMap<_, BTreeSet<_>>, link| {
            let [from, to] = link.ends();
            adjacency.entry(from).or_default().insert(to);
            adjacency
        })
    }

    pub(crate) fn slices(&self) -> impl Iterator<Item = &Slice<'found>> + '_ {
        self.slices.values()
    }

    pub(crate) fn slice(&self, cluster: Cluster<'found>) -> Option<&Slice<'found>> {
        self.slices.get(&cluster)
    }

    pub(crate) fn all(&self) -> impl Iterator<Item = Member<'found>> + '_ {
        self.sites.keys().copied()
    }

    pub(crate) fn adjacent(&self, member: Member<'found>, toward: Toward) -> impl Iterator<Item = Member<'found>> + '_ {
        self.toward(toward).get(&member).into_iter().flatten().copied()
    }

    pub(crate) fn links_within(&self, members: &BTreeSet<Member<'found>>, toward: Toward) -> Vec<Link<'found>> {
        members
            .iter()
            .flat_map(|member| self.adjacent(*member, toward).map(|used| Link::new(*member, used)))
            .filter(|link| link.ends().iter().all(|end| members.contains(end)))
            .collect()
    }

    pub(crate) fn first_site(&self, members: &BTreeSet<Member<'found>>) -> Option<Site<'file>> {
        members.iter().filter_map(|member| self.sites.get(member).copied()).min()
    }
}
