use std::collections::BTreeSet;
use std::fmt;

use lint_core::{Fix, Headline, Report, Violation};

use super::cluster::{Cluster, Level, Part};
use super::groups::Groups;
use super::member::Member;
use super::{ClusterLint, Containment};

#[derive(Debug)]
pub struct ConnectedClusters;

impl ClusterLint for ConnectedClusters {
    fn report_clusters<'file>(view: &Containment<'_, 'file>) -> Report<'file> {
        Report::new(
            Headline::new(
                "report only, each crate and each directory module is one connected piece of the type and trait graph: types and traits are linked by fields, variants, bounds, trait impls and the signatures of their methods, and a cluster whose items fall into groups with no link between them is several clusters in one",
            ),
            view.clusters()
                .into_iter()
                .filter_map(|cluster| Divided::of(view, cluster))
                .filter_map(|divided| view.first_site(&divided.members).map(|site| Violation::new(site, &divided)))
                .collect(),
        )
    }
}

#[derive(Debug)]
struct Divided<'found> {
    cluster: Cluster<'found>,
    members: BTreeSet<Member<'found>>,
    groups: Groups<'found>,
}

impl<'found> Divided<'found> {
    fn of(view: &Containment<'found, '_>, cluster: Cluster<'found>) -> Option<Divided<'found>> {
        let members = view.members(cluster);
        let nested = members.iter().find_map(|member| Part::of(cluster, member.cluster()).nested());
        let judged = match cluster.level() {
            Level::Crate => Some(cluster),
            Level::Module => nested.map(|_| cluster),
            Level::Workspace => None,
        };
        let pairs = members.iter().flat_map(|member| {
            view.uses_of(*member).filter(|used| members.contains(used)).map(|used| (*member, used)).collect::<Vec<_>>()
        });
        let groups = judged.and_then(|_| Groups::linked(&members, pairs).split())?;
        Some(Divided { cluster, members, groups })
    }
}

impl fmt::Display for Divided<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} falls into {} with no link between them", self.cluster, self.groups)
    }
}

impl lint_core::Finding for Divided<'_> {
    fn fix(&self) -> Fix {
        Fix::new("split it along the groups, or link them through a type they share")
    }
}
