use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use lint_core::{Fix, Headline, Violation};

use super::cluster::{Cluster, Level};
use super::groups::Groups;
use super::link::Link;
use super::member::Member;
use super::{ClusterLint, Containment};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Linking {
    Consumers,
    Internals,
}

impl Linking {
    const fn headline(self) -> Headline {
        Headline::new(match self {
            Linking::Consumers => {
                "report only, each crate and module has one interface by its users: its exports are the types and traits used from outside it, two exports are linked when one outside type or trait uses both, and the linked exports form one group"
            }
            Linking::Internals => {
                "report only, each crate and module has one interface by its internals: its exports are the types and traits used from outside it, two exports are linked when both reach one private type or trait of it, and the linked exports form one group"
            }
        })
    }

    fn violations<'file>(self, view: &Containment<'_, 'file>) -> Vec<Violation<'file>> {
        view.clusters()
            .into_iter()
            .filter_map(|cluster| Interface::split_in(view, cluster, self))
            .filter_map(|split| view.first_site(&split.exports).map(|site| Violation::new(site, &split)))
            .collect()
    }

    fn links<'found>(
        self,
        view: &Containment<'found, '_>,
        cluster: Cluster<'found>,
        exports: &BTreeSet<Member<'found>>,
    ) -> Vec<Link<'found>> {
        let members = view.members(cluster);
        let sharing = exports
            .iter()
            .flat_map(|export| match self {
                Linking::Consumers => view.outside_users(cluster, *export),
                Linking::Internals => view.private_reached(&members, exports, *export),
            })
            .fold(BTreeMap::new(), |mut sharing: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>, link| {
                let [shared, export] = link.ends();
                sharing.entry(shared).or_default().insert(export);
                sharing
            });
        sharing.values().flat_map(Link::star).collect()
    }
}

#[derive(Debug)]
struct Interface<'found> {
    cluster: Cluster<'found>,
    linking: Linking,
    exports: BTreeSet<Member<'found>>,
    groups: Groups<'found>,
}

impl<'found> Interface<'found> {
    fn split_in(
        view: &Containment<'found, '_>,
        cluster: Cluster<'found>,
        linking: Linking,
    ) -> Option<Interface<'found>> {
        let judged = match cluster.level() {
            Level::Crate | Level::Module => Some(cluster),
            Level::Workspace => None,
        }?;
        let exports = view.exports(judged, &view.members(judged));
        let groups = Groups::linked(&exports, linking.links(view, judged, &exports).into_iter()).split()?;
        Some(Interface { cluster: judged, linking, exports, groups })
    }
}

impl fmt::Display for Interface<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let by = match self.linking {
            Linking::Consumers => "no outside user joins",
            Linking::Internals => "no private item joins",
        };
        write!(formatter, "the exports of {} fall into {} that {by}", self.cluster, self.groups)
    }
}

impl lint_core::Finding for Interface<'_> {
    fn fix(&self) -> Fix {
        Fix::new(match self.linking {
            Linking::Consumers => "split it along the groups, one part for each kind of user",
            Linking::Internals => "split it along the groups, since they share no implementation",
        })
    }
}

#[derive(Debug)]
pub struct InterfaceByConsumers;

impl ClusterLint for InterfaceByConsumers {
    const HEADLINE: Headline = Linking::Consumers.headline();

    fn violations<'file>(view: &Containment<'_, 'file>) -> Vec<Violation<'file>> {
        Linking::Consumers.violations(view)
    }
}

#[derive(Debug)]
pub struct InterfaceByInternals;

impl ClusterLint for InterfaceByInternals {
    const HEADLINE: Headline = Linking::Internals.headline();

    fn violations<'file>(view: &Containment<'_, 'file>) -> Vec<Violation<'file>> {
        Linking::Internals.violations(view)
    }
}
