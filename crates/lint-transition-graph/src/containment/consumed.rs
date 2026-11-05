use std::collections::BTreeSet;
use std::fmt;

use lint_core::Fix;

use super::cluster::Cluster;
use super::groups::Groups;
use super::link::Link;
use super::member::Member;
use super::{Containment, Split, Toward};

#[derive(Debug)]
pub(super) struct Interface<'view, 'found> {
    cluster: Cluster<'found>,
    exports: &'view BTreeSet<Member<'found>>,
    groups: Groups<'found>,
}

impl<'view, 'found> Split<'view, 'found> for Interface<'view, 'found> {
    fn found_in(view: &'view Containment<'found, '_>, cluster: Cluster<'found>) -> Option<Interface<'view, 'found>> {
        let judged = cluster.unit().map(|_| cluster)?;
        let cut = view.slice(judged)?.cut();
        let exports = cut.exports();
        let sharing = Containment::adjacency(cut.outgoing().iter().copied());
        let signed = view.links_within(exports, Toward::Signed);
        let links = sharing.values().flat_map(Link::star).chain(signed);
        let groups = Groups::linked(exports, links).split()?;
        Some(Interface { cluster: judged, exports, groups })
    }

    fn sited(&self) -> &BTreeSet<Member<'found>> {
        self.exports
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
