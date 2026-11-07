use std::collections::BTreeSet;
use std::fmt;

use lint_core::{Fix, Headline, Violation};

use super::cluster::Cluster;
use super::groups::Groups;
use super::link::Link;
use super::member::Member;
use super::{ClusterLint, Containment};

#[derive(Debug)]
struct Interface<'view, 'found> {
    cluster: Cluster<'found>,
    exports: &'view BTreeSet<Member<'found>>,
    groups: Groups<'found>,
}

impl<'view, 'found> Interface<'view, 'found> {
    fn split_in(view: &'view Containment<'found, '_>, cluster: Cluster<'found>) -> Option<Self> {
        let judged = cluster.krate().map(|_| cluster)?;
        let cut = view.slice(judged)?.cut();
        let exports = cut.exports();
        let sharing = Containment::adjacency(cut.outgoing().iter().copied());
        let links = sharing.values().flat_map(Link::star);
        let groups = Groups::linked(exports, links).split()?;
        Some(Interface { cluster: judged, exports, groups })
    }
}

impl fmt::Display for Interface<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "the exports of {} fall into {} that no outside user joins", self.cluster, self.groups)
    }
}

impl lint_core::Finding for Interface<'_, '_> {
    fn fix(&self) -> Fix {
        Fix::new("split it along the groups, one part for each kind of user")
    }
}

#[derive(Debug)]
pub struct InterfaceByConsumers;

impl ClusterLint for InterfaceByConsumers {
    const HEADLINE: Headline = Headline::new(
        "report only, each crate and module has one interface by its users: its exports are the types and traits used from outside it, two exports are linked when one outside type or trait uses both, and the linked exports form one group",
    );

    fn violations<'file>(view: &Containment<'_, 'file>) -> Vec<Violation<'file>> {
        view.slices()
            .filter_map(|slice| Interface::split_in(view, slice.cluster()))
            .filter_map(|split| view.first_site(split.exports).map(|site| Violation::new(site, &split)))
            .collect()
    }
}
