mod cluster;
mod connected;
mod declaration;
mod depth;
mod groups;
mod interface;
mod layered;
mod link;
mod member;
mod placed;
mod resolver;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use lint_core::{Headline, Report, Site, SourceFile, Violation};
use lint_item_graph::{ItemGraph, Traversal};
use lint_layout::Name;
use lint_placement::CrateName;
use transition_facts::Transitions;
use type_facts::Facts;

use cluster::Cluster;
use declaration::Declaration;
use link::Link;
use member::Member;
use resolver::Resolver;

pub use connected::ConnectedClusters;
pub use depth::DeepClusters;
pub use interface::{InterfaceByConsumers, InterfaceByInternals};
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
}

#[derive(Debug)]
pub struct Containment<'found, 'file> {
    sites: BTreeMap<Member<'found>, Site<'file>>,
    uses: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
    users: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
    defines: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
}

impl<'found, 'file> Containment<'found, 'file> {
    #[must_use]
    pub fn clustered(
        files: &'found [SourceFile],
        facts: &'found Facts<'file>,
        transitions: &'found Transitions<'file>,
    ) -> Containment<'found, 'file> {
        let modules: BTreeMap<&Path, &[Name]> = files
            .iter()
            .filter_map(SourceFile::shipped)
            .map(|file| (file.path(), file.placement().module().segments()))
            .collect();
        let declarations = Declaration::all(facts);
        let declared = declarations.iter().fold(BTreeMap::new(), |mut declared: BTreeMap<_, Vec<_>>, declaration| {
            declared.entry(*AsRef::<Member<'found>>::as_ref(declaration)).or_default().push(declaration);
            declared
        });
        let placed: BTreeMap<Member<'found>, &Declaration<'found, 'file>> = declared
            .into_values()
            .filter_map(|twins| twins.first().copied().filter(|_| twins.len() == 1))
            .filter_map(|declaration| {
                let site: &Site<'file> = declaration.as_ref();
                let named: &Member<'found> = declaration.as_ref();
                modules.get(site.path()).and_then(|segments| named.within(segments)).map(|member| (member, declaration))
            })
            .collect();
        let resolver = &Resolver::of_members(placed.keys().copied());
        let defined: BTreeSet<Link<'found>> = placed
            .iter()
            .flat_map(|(member, declaration)| declaration.links(*member, resolver))
            .filter_map(Link::apart)
            .collect();
        let signed = transitions.all().flat_map(|transition| resolver.signature(transition));
        let crates = files.iter().filter_map(SourceFile::shipped).fold(
            BTreeMap::new(),
            |mut crates: BTreeMap<&CrateName, Vec<&SourceFile>>, file| {
                crates.entry(file.placement().package().crate_name()).or_default().push(file);
                crates
            },
        );
        let bodies = crates.iter().flat_map(|(krate, files)| resolver.bodies(krate, &ItemGraph::of_files(files)));
        let links: BTreeSet<Link<'found>> =
            defined.iter().copied().chain(signed).chain(bodies).filter_map(Link::apart).collect();
        Containment {
            sites: placed
                .iter()
                .map(|(member, declaration)| (*member, *AsRef::<Site<'file>>::as_ref(*declaration)))
                .collect(),
            uses: Self::adjacency(links.iter().copied()),
            users: Self::adjacency(links.iter().copied().map(Link::reversed)),
            defines: Self::adjacency(defined.into_iter()),
        }
    }

    #[must_use]
    pub fn report<Check: ClusterLint>(&self) -> Report<'file> {
        Report::new(Check::HEADLINE, Check::violations(self))
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

    pub(crate) fn clusters(&self) -> BTreeSet<Cluster<'found>> {
        self.sites.keys().flat_map(|member| member.cluster().lineage()).collect()
    }

    pub(crate) fn members(&self, cluster: Cluster<'found>) -> BTreeSet<Member<'found>> {
        self.sites.keys().copied().filter(|member| cluster.holding(member.cluster()).is_some()).collect()
    }

    pub(crate) fn all(&self) -> impl Iterator<Item = Member<'found>> + '_ {
        self.sites.keys().copied()
    }

    pub(crate) fn adjacent(&self, member: Member<'found>, toward: Toward) -> impl Iterator<Item = Member<'found>> + '_ {
        let adjacency = match toward {
            Toward::Used => &self.uses,
            Toward::Users => &self.users,
            Toward::Defined => &self.defines,
        };
        adjacency.get(&member).into_iter().flatten().copied()
    }

    pub(crate) fn links_within(&self, members: &BTreeSet<Member<'found>>, toward: Toward) -> Vec<Link<'found>> {
        members
            .iter()
            .flat_map(|member| self.adjacent(*member, toward).map(|used| Link::new(*member, used)))
            .filter(|link| link.ends().iter().all(|end| members.contains(end)))
            .collect()
    }

    pub(crate) fn reach(&self, start: Member<'found>, within: &BTreeSet<Member<'found>>) -> Vec<Member<'found>> {
        let step = |member: Member<'found>| self.adjacent(member, Toward::Used).filter(|used| within.contains(used));
        Traversal::trees([start], |member| step(member).collect::<Vec<_>>().into_iter()).into_iter().flatten().collect()
    }

    pub(crate) fn outside_users(&self, cluster: Cluster<'found>, export: Member<'found>) -> Vec<Link<'found>> {
        self.adjacent(export, Toward::Users)
            .filter(|user| cluster.holding(user.cluster()).is_none())
            .map(|user| Link::new(user, export))
            .collect()
    }

    pub(crate) fn private_reached(
        &self,
        members: &BTreeSet<Member<'found>>,
        exports: &BTreeSet<Member<'found>>,
        export: Member<'found>,
    ) -> Vec<Link<'found>> {
        self.reach(export, members)
            .into_iter()
            .filter(|reached| !exports.contains(reached))
            .map(|reached| Link::new(reached, export))
            .collect()
    }

    pub(crate) fn first_site(&self, members: &BTreeSet<Member<'found>>) -> Option<Site<'file>> {
        members.iter().filter_map(|member| self.sites.get(member).copied()).min()
    }

    pub(crate) fn exports(
        &self,
        cluster: Cluster<'found>,
        members: &BTreeSet<Member<'found>>,
    ) -> BTreeSet<Member<'found>> {
        members
            .iter()
            .copied()
            .filter(|member| {
                self.adjacent(*member, Toward::Users).any(|user| cluster.holding(user.cluster()).is_none())
            })
            .collect()
    }
}
