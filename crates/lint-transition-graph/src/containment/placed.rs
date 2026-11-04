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
        "report only, each type and trait sits in the lowest cluster that holds all its users: the users are the types and traits that hold it in a field, variant or bound, implement it, or name it in a method signature, and the lowest cluster is the deepest crate or module whose subtree holds them all",
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
        let lowest = view.users_within(member)?;
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
        Fix::new("move it into the lowest cluster that holds all its users")
    }
}
