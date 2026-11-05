use std::collections::BTreeSet;
use std::fmt;

use lint_core::{Fix, Headline, Violation};

use super::cluster::Cluster;
use super::member::Member;
use super::{ClusterLint, Containment};

#[derive(Debug)]
pub struct LowestCluster;

impl ClusterLint for LowestCluster {
    const HEADLINE: Headline = Headline::new(
        "each type and trait sits in the lowest module of its crate that holds all its users: the users are the types, traits and free functions of its crate that hold it in a field, variant or bound, implement it, name it in a method signature or use it in a body, the functions of a binary root are not users, and the lowest module is the deepest one whose subtree holds them all; a type with users in other crates is placed by its crate",
    );

    fn violations<'file>(view: &Containment<'_, 'file>) -> Vec<Violation<'file>> {
        view.all()
            .filter_map(|member| Misplaced::found(view, member))
            .filter_map(|misplaced| {
                view.first_site(&BTreeSet::from([misplaced.member])).map(|site| Violation::new(site, &misplaced))
            })
            .collect()
    }
}

#[derive(Debug)]
struct Misplaced<'found> {
    member: Member<'found>,
    lowest: Cluster<'found>,
}

impl<'found> Misplaced<'found> {
    fn found(view: &Containment<'found, '_>, member: Member<'found>) -> Option<Misplaced<'found>> {
        let lowest = view
            .users_within(member)
            .filter(|lowest| lowest.krate().is_some_and(|krate| member.cluster().krate() == Some(krate)))?;
        let outside = lowest.holding(member.cluster()).map_or(Some(lowest), |_| None)?;
        Some(Misplaced { member, lowest: outside })
    }
}

impl fmt::Display for Misplaced<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} is used only within {}", self.member, self.lowest)
    }
}

impl lint_core::Finding for Misplaced<'_> {
    fn fix(&self) -> Fix {
        Fix::new("move it into the lowest module that holds all its users")
    }
}
