use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Fix;
use newtype::Counter;

use super::behaviour::{Behaviour, Move};
use crate::count::Count;
use crate::end::Node;
use crate::graph::Graph;

#[derive(Debug)]
struct Member<'graph> {
    node: &'graph Node,
    site: &'graph Site,
}

#[derive(Debug)]
pub(super) struct Group<'graph> {
    members: Vec<Member<'graph>>,
    moves: Vec<&'graph Move<'graph>>,
}

impl<'graph> Group<'graph> {
    pub(super) fn alike(
        members: Vec<&'graph Node>,
        graph: &'graph Graph<'_>,
        behaviour: &'graph Behaviour<'graph>,
    ) -> Option<Group<'graph>> {
        let moves: Vec<&'graph Move<'graph>> =
            members.first().and_then(|node| behaviour.moves_of(node)).into_iter().flatten().collect();
        let members: Vec<Member<'graph>> =
            members.into_iter().filter_map(|node| graph.nodes().get(node).map(|site| Member { node, site })).collect();
        (members.len() > 1 && moves.len() > 1).then_some(Group { members, moves })
    }

    pub(super) fn ranked(mut groups: Vec<Group<'graph>>) -> Vec<Group<'graph>> {
        groups.sort_by(|left, right| {
            right.members.len().cmp(&left.members.len()).then_with(|| left.first().cmp(&right.first()))
        });
        groups
    }

    fn first(&self) -> Option<&'graph Site> {
        self.members.iter().map(|member| member.site).min()
    }
}

impl From<&Group<'_>> for Violation {
    fn from(group: &Group<'_>) -> Violation {
        let site = group.first().map_or_else(|| Site::of_file(std::path::Path::new(".")), Clone::clone);
        Violation::new(site, group)
    }
}

impl fmt::Display for Group<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let members: Vec<String> =
            self.members.iter().map(|member| format!("{} at {}", member.node, member.site)).collect();
        let moves: Vec<String> = self.moves.iter().map(ToString::to_string).collect();
        write!(
            formatter,
            "{} types behave alike, each answering {}: {}",
            self.members.len(),
            moves.join(", "),
            members.join(", ")
        )
    }
}

impl lint_layout::Finding for Group<'_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "merge the group into one type, or one generic type over what differs, or give each a behaviour of its own",
        )
    }
}

#[derive(Debug)]
pub(super) struct Tally {
    nodes: Count,
    groups: Count,
    grouped: Count,
}

impl Tally {
    pub(super) fn counted(nodes: Count, groups: &[Group<'_>]) -> Tally {
        Tally {
            nodes,
            groups: Count::new(groups.len()),
            grouped: Count::total(groups.iter().map(|group| Count::new(group.members.len()))),
        }
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "workspace: {} domain types, {} groups of bisimilar types holding {}",
            self.nodes, self.groups, self.grouped
        )
    }
}

impl lint_layout::Finding for Tally {
    fn fix(&self) -> Fix {
        Fix::new("merge each group of bisimilar types into one type or one generic")
    }
}
