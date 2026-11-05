use std::collections::BTreeSet;
use std::fmt;

use lint_core::Fix;

use super::cluster::{Cluster, Level, Part};
use super::groups::Groups;
use super::member::Member;
use super::{Containment, Split, Toward};

#[derive(Debug)]
pub(super) struct Divided<'view, 'found> {
    cluster: Cluster<'found>,
    members: &'view BTreeSet<Member<'found>>,
    groups: Groups<'found>,
}

impl<'view, 'found> Split<'view, 'found> for Divided<'view, 'found> {
    fn found_in(view: &'view Containment<'found, '_>, cluster: Cluster<'found>) -> Option<Divided<'view, 'found>> {
        let slice = view.slice(cluster)?;
        let members = slice.members();
        let nested = members.iter().find_map(|member| Part::toward(cluster, member.cluster()).nested());
        let judged = Self::judged(cluster, nested);
        let groups = judged.and_then(|_| slice.components(view, Toward::Used))?;
        Some(Divided { cluster, members, groups })
    }

    fn sited(&self) -> &BTreeSet<Member<'found>> {
        self.members
    }
}

impl<'found> Divided<'_, 'found> {
    fn judged(cluster: Cluster<'found>, nested: Option<Cluster<'found>>) -> Option<Cluster<'found>> {
        match cluster.level() {
            Level::Crate => Some(cluster),
            Level::Module => nested.map(|_| cluster),
            Level::Workspace => None,
        }
    }
}

impl fmt::Display for Divided<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} falls into {} with no link between them", self.cluster, self.groups)
    }
}

impl lint_core::Finding for Divided<'_, '_> {
    fn fix(&self) -> Fix {
        Fix::new("split it along the groups, or link them through a type they share")
    }
}
