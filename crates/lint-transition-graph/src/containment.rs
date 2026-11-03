mod cluster;
mod connected;
mod declaration;
mod depth;
mod groups;
mod interface;
mod layered;
mod member;
mod placed;
mod resolver;

use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::path::Path;

use lint_core::{Lint, Report, Site, SourceFile};
use lint_layout::Name;
use lint_placement::CrateName;
use transition_facts::Transitions;
use type_facts::Facts;

use cluster::Cluster;
use declaration::Declaration;
use member::Member;
use resolver::Resolver;

pub use connected::ConnectedClusters;
pub use depth::DeepClusters;
pub use interface::{InterfaceByConsumers, InterfaceByInternals};
pub use layered::AcyclicQuotients;
pub use placed::LowestCluster;

pub trait ClusterLint {
    fn report_clusters<'file>(view: &Containment<'_, 'file>) -> Report<'file>;
}

#[derive(Debug)]
pub struct OnClusters<Check>(PhantomData<Check>);

impl<Check: ClusterLint> Lint for OnClusters<Check> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let facts = Facts::of_files(files);
        let transitions = Transitions::of_files(files);
        Check::report_clusters(&Containment::of(files, &facts, &transitions))
    }
}

#[derive(Debug)]
pub struct Containment<'found, 'file> {
    sites: BTreeMap<Member<'found>, Site<'file>>,
    uses: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
    users: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
}

impl<'found, 'file> Containment<'found, 'file> {
    #[must_use]
    pub fn of(
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
            declared.entry(declaration.key()).or_default().push(declaration);
            declared
        });
        let placed: Vec<(Member<'found>, &Declaration<'found, 'file>)> = declared
            .into_values()
            .filter_map(|twins| twins.first().copied().filter(|_| twins.len() == 1))
            .filter_map(|declaration| {
                modules.get(declaration.site().path()).map(|segments| (declaration.placed(*segments), declaration))
            })
            .collect();
        let resolver = Resolver::of_members(placed.iter().map(|(member, _)| *member));
        let defined = placed
            .iter()
            .flat_map(|(member, declaration)| declaration.used(&resolver).into_iter().map(|used| (*member, used)));
        let signed = transitions.all().flat_map(|transition| {
            let from: &CrateName = transition.as_ref();
            let holder = transition.context().subject().and_then(|subject| resolver.endpoint(from, subject));
            let ends: Vec<Member<'found>> = transition
                .takes()
                .iter()
                .chain([transition.gives().endpoint()])
                .filter_map(|end| resolver.endpoint(from, end))
                .collect();
            ends.into_iter().filter_map(move |end| holder.map(|holder| (holder, end)))
        });
        let edges: BTreeSet<(Member<'found>, Member<'found>)> =
            defined.chain(signed).filter(|(user, used)| user != used).collect();
        Containment {
            sites: placed.iter().map(|(member, declaration)| (*member, declaration.site())).collect(),
            uses: Self::adjacency(edges.iter().copied()),
            users: Self::adjacency(edges.iter().map(|(user, used)| (*used, *user))),
        }
    }

    fn adjacency(
        edges: impl Iterator<Item = (Member<'found>, Member<'found>)>,
    ) -> BTreeMap<Member<'found>, BTreeSet<Member<'found>>> {
        edges.fold(BTreeMap::new(), |mut adjacency: BTreeMap<_, BTreeSet<_>>, (from, to)| {
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

    pub(crate) fn uses_of(&self, member: Member<'found>) -> impl Iterator<Item = Member<'found>> + '_ {
        self.uses.get(&member).into_iter().flatten().copied()
    }

    pub(crate) fn users_of(&self, member: Member<'found>) -> impl Iterator<Item = Member<'found>> + '_ {
        self.users.get(&member).into_iter().flatten().copied()
    }

    pub(crate) fn site_of(&self, member: Member<'found>) -> Option<Site<'file>> {
        self.sites.get(&member).copied()
    }

    pub(crate) fn first_site(&self, members: &BTreeSet<Member<'found>>) -> Option<Site<'file>> {
        members.iter().filter_map(|member| self.site_of(*member)).min()
    }

    pub(crate) fn exports(&self, cluster: Cluster<'found>, members: &BTreeSet<Member<'found>>) -> BTreeSet<Member<'found>> {
        members
            .iter()
            .copied()
            .filter(|member| self.users_of(*member).any(|user| cluster.holding(user.cluster()).is_none()))
            .collect()
    }
}
