mod ask;
mod cluster;
mod connected;
mod declaration;
mod groups;
mod housing;
mod interface;
mod layered;
mod link;
mod located;
mod member;
mod outsider;
mod placed;
mod resolver;
mod slice;
mod unit;

use std::collections::{BTreeMap, BTreeSet};

use lint_core::{Headline, Report, Site, SourceFile, Violation};
use lint_item_graph::{ItemGraph, ReferencedNames};
use lint_layout::Name;
use transition_facts::Transitions;
use type_facts::{Facts, Implementation};

use cluster::{Cluster, Level};
use declaration::Declaration;
use housing::Housing;
use link::Link;
use located::Located;
use member::Member;
use outsider::Outsider;
use resolver::{Functions, Resolver, StandIns};
use slice::Slice;
use unit::Unit;

pub use ask::{Answer, Ask, Listed};
pub use connected::{ConnectedClusters, ConnectedDirectories};
pub use interface::InterfaceByConsumers;
pub use layered::AcyclicQuotients;
pub use placed::LowestCluster;

pub trait ClusterLint {
    const HEADLINE: Headline;

    fn violations<'file>(view: &Containment<'_, 'file>) -> Vec<Violation<'file>>;
}

pub(crate) trait Split<'view, 'found>: lint_core::Finding + Sized {
    fn found_in(view: &'view Containment<'found, '_>, cluster: Cluster<'found>) -> Option<Self>;

    fn sited(&self) -> &BTreeSet<Member<'found>>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Toward {
    Used,
    Users,
    Defined,
    Implementers,
    Signed,
}

#[derive(Debug)]
pub struct Containment<'found, 'file> {
    sites: BTreeMap<Member<'found>, Site<'file>>,
    uses: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
    users: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
    defines: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
    implementers: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
    signed: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
    outsiders: BTreeMap<Member<'found>, BTreeSet<Outsider<'found, 'file>>>,
    slices: BTreeMap<Cluster<'found>, Slice<'found>>,
}

impl<'found, 'file> Containment<'found, 'file> {
    #[must_use]
    pub fn clustered(
        files: &'found [SourceFile],
        facts: &'found Facts<'file>,
        transitions: &'found Transitions<'file>,
    ) -> Containment<'found, 'file> {
        let housing = Housing::by_target(files);
        let homes = housing.homes();
        let declarations = Declaration::all(facts);
        let placed: BTreeMap<Member<'found>, &Declaration<'found, 'file>> = declarations
            .iter()
            .filter_map(|declaration| {
                let site: &Site<'file> = declaration.as_ref();
                let named: &Member<'found> = declaration.as_ref();
                homes.get(site.path()).map(|home| (named.within(*home), declaration))
            })
            .collect();
        let named: Vec<(&SourceFile, ReferencedNames)> =
            housing.files().iter().map(|file| (*file, ReferencedNames::of_file(file))).collect();
        let crates = named.iter().fold(
            BTreeMap::new(),
            |mut crates: BTreeMap<Unit<'found>, Vec<&ReferencedNames>>, (file, names)| {
                housing.unit(file.path()).into_iter().for_each(|unit| crates.entry(unit).or_default().push(names));
                crates
            },
        );
        let graphs: Vec<(Unit<'found>, ItemGraph)> = crates
            .into_iter()
            .map(|(krate, names)| {
                let merged = names.into_iter().cloned().fold(ReferencedNames::default(), ReferencedNames::merged);
                (krate, merged.into_graph())
            })
            .collect();
        let stand_ins =
            graphs.iter().fold(StandIns::default(), |stand_ins, (unit, graph)| stand_ins.with_graph(*unit, graph));
        let functions = Functions::declared(transitions, &homes);
        let resolver = &Resolver::of_members(placed.keys().copied(), &functions, &housing, stand_ins);
        let located: Vec<Located<'found, 'file>> =
            facts.outside_implementations().map(|outside| Located::resolved_by(resolver, outside)).collect();
        let implemented: BTreeSet<Link<'found>> = placed
            .iter()
            .flat_map(|(member, declaration)| {
                declaration.implemented(*member, resolver).into_iter().map(|used| Link::new(*member, used))
            })
            .chain(located.iter().flat_map(Located::links))
            .filter_map(Link::apart)
            .collect();
        let defined: BTreeSet<Link<'found>> = placed
            .iter()
            .flat_map(|(member, declaration)| declaration.held(*member, resolver))
            .filter_map(Link::apart)
            .chain(implemented.iter().copied())
            .collect();
        let outsiders = located.iter().flat_map(Located::outsiders).fold(
            BTreeMap::new(),
            |mut outsiders: BTreeMap<Member<'found>, BTreeSet<Outsider<'found, 'file>>>, outsider| {
                outsiders.entry(outsider.holder()).or_default().insert(outsider);
                outsiders
            },
        );
        let argued = placed.iter().flat_map(|(member, declaration)| {
            let implementations: &[Implementation<'file>] = declaration.as_ref();
            implementations
                .iter()
                .flat_map(|implementation| resolver.argument(implementation))
                .map(|argued| Link::new(*member, argued))
        });
        let signed: BTreeSet<Link<'found>> = transitions
            .all()
            .flat_map(|transition| resolver.signature(transition))
            .chain(argued)
            .chain(defined.iter().copied())
            .filter_map(Link::apart)
            .collect();
        let bodies = named.iter().flat_map(|(file, names)| resolver.bodies(file, names));
        let links: BTreeSet<Link<'found>> = signed.iter().copied().chain(bodies).filter_map(Link::apart).collect();
        let users = Self::adjacency(links.iter().copied().map(Link::reversed));
        Containment {
            slices: Slice::all(placed.keys().copied(), &users),
            sites: placed
                .iter()
                .map(|(member, declaration)| (*member, *AsRef::<Site<'file>>::as_ref(*declaration)))
                .collect(),
            uses: Self::adjacency(links.iter().copied()),
            users,
            defines: Self::adjacency(defined.into_iter()),
            implementers: Self::adjacency(implemented.into_iter().map(Link::reversed)),
            signed: Self::adjacency(signed.into_iter()),
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
            Toward::Signed => &self.signed,
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
            .filter(|toward| *toward == Toward::Implementers)
            .into_iter()
            .flat_map(move |_| self.outsiders.get(&member).into_iter().flatten().copied())
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

    pub(crate) fn splits<'view, Found: Split<'view, 'found>>(&'view self, levels: &[Level]) -> Vec<Violation<'file>> {
        self.slices()
            .filter(|slice| levels.contains(&slice.cluster().level()))
            .filter_map(|slice| Found::found_in(self, slice.cluster()))
            .filter_map(|found| self.first_site(found.sited()).map(|site| Violation::new(site, &found)))
            .collect()
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
