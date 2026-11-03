use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Fix;
use newtype::Counter;

use super::behaviour::{Behaviour, Move, SetAside};
use crate::count::Count;
use crate::end::Node;
use crate::graph::Graph;

#[derive(Debug)]
struct Member<'graph, 'file> {
    node: &'graph Node<'graph>,
    site: Site<'file>,
}

impl<'graph, 'file> Member<'graph, 'file> {
    fn placed(node: &'graph Node<'graph>, graph: &'graph Graph<'_, 'file>) -> Option<Member<'graph, 'file>> {
        graph.nodes().get(node).map(|site| Member { node, site: *site })
    }
}

#[derive(Debug)]
pub(super) struct Group<'graph, 'file> {
    members: Vec<Member<'graph, 'file>>,
    moves: Vec<&'graph Move<'graph>>,
}

impl<'graph, 'file> Group<'graph, 'file> {
    pub(super) fn ranked(
        grouped: impl IntoIterator<Item = Vec<&'graph Node<'graph>>>,
        graph: &'graph Graph<'_, 'file>,
        behaviour: &'graph Behaviour<'graph>,
    ) -> Vec<Group<'graph, 'file>> {
        let mut groups: Vec<Group<'graph, 'file>> = grouped
            .into_iter()
            .filter_map(|nodes| {
                let moves: Vec<&'graph Move<'graph>> =
                    nodes.first().and_then(|node| behaviour.moves_of(node)).into_iter().flatten().collect();
                let members: Vec<Member<'graph, 'file>> =
                    nodes.into_iter().filter_map(|node| Member::placed(node, graph)).collect();
                (members.len() > 1 && moves.len() > 1).then_some(Group { members, moves })
            })
            .collect();
        groups.sort_by(|left, right| {
            right.members.len().cmp(&left.members.len()).then_with(|| left.first().cmp(&right.first()))
        });
        groups
    }

    fn first(&self) -> Option<Site<'file>> {
        self.members.iter().map(|member| member.site).min()
    }
}

impl<'file> From<&Group<'_, 'file>> for Violation<'file> {
    fn from(group: &Group<'_, 'file>) -> Violation<'file> {
        let site = group.first().unwrap_or_else(|| Site::of_file(std::path::Path::new(".")));
        Violation::new(site, group)
    }
}

impl fmt::Display for Group<'_, '_> {
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

impl lint_layout::Finding for Group<'_, '_> {
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
    set_aside: SetAside,
}

impl Tally {
    pub(super) fn counted(nodes: Count, behaviour: &Behaviour<'_>, groups: &[Group<'_, '_>]) -> Tally {
        Tally {
            nodes,
            set_aside: behaviour.set_aside(),
            groups: Count::new(groups.len()),
            grouped: Count::total(groups.iter().map(|group| Count::new(group.members.len()))),
        }
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "workspace: {} domain types, {} groups of bisimilar types holding {}; set aside as grouped under their interface: {}",
            self.nodes, self.groups, self.grouped, self.set_aside
        )
    }
}

impl lint_layout::Finding for Tally {
    fn fix(&self) -> Fix {
        Fix::new("merge each group of bisimilar types into one type or one generic")
    }
}
