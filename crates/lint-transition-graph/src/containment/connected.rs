use std::collections::BTreeSet;
use std::fmt;

use lint_core::{Fix, Headline, Violation};

use super::cluster::{Cluster, Level, Part};
use super::groups::Groups;
use super::member::Member;
use super::{ClusterLint, Containment, Split, Toward};
use crate::Covered;

#[derive(Debug)]
pub struct ConnectedClusters;

impl ClusterLint for ConnectedClusters {
    const HEADLINE: Headline = Headline::new(
        "each crate is one connected piece of the type and trait graph: types and traits are linked by fields, variants, bounds, trait impls, the signatures of their methods and the names their bodies use, and a crate whose items fall into groups with no link between them is several crates in one",
    );

    fn violations<'file>(view: &Containment<'_, 'file>) -> Vec<Violation<'file>> {
        view.splits::<Divided<'_, '_>>(Level::Crate)
    }
}

impl Covered for ConnectedClusters {
    type Coverage = ConnectedDirectories;
}

#[derive(Debug)]
pub struct ConnectedDirectories;

impl ClusterLint for ConnectedDirectories {
    const HEADLINE: Headline = Headline::new(
        "report only, each directory module is one connected piece of the type and trait graph: types and traits are linked by fields, variants, bounds, trait impls and the signatures of their methods, and a directory module whose items fall into groups with no link between them is several modules in one",
    );

    fn violations<'file>(view: &Containment<'_, 'file>) -> Vec<Violation<'file>> {
        view.splits::<Divided<'_, '_>>(Level::Module)
    }
}

#[derive(Debug)]
struct Divided<'view, 'found> {
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
