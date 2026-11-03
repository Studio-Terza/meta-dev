use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Fix;

use crate::count::Count;
use crate::end::Node;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Reach {
    Reached,
    Unreached,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Usage {
    Consumed,
    Unconsumed,
}

#[derive(Debug)]
pub(super) struct Liveness<'found, 'file> {
    node: &'found Node<'found>,
    site: Site<'file>,
    reach: Reach,
    usage: Usage,
}

impl<'found, 'file> Liveness<'found, 'file> {
    pub(super) fn judged(
        node: &'found Node<'found>,
        site: Site<'file>,
        reach: Reach,
        usage: Usage,
    ) -> Option<Liveness<'found, 'file>> {
        match (reach, usage) {
            (Reach::Reached, Usage::Consumed) => None,
            _ => Some(Liveness { node, site, reach, usage }),
        }
    }
}

impl<'file> From<&Liveness<'_, 'file>> for Violation<'file> {
    fn from(live: &Liveness<'_, 'file>) -> Violation<'file> {
        Violation::new(live.site, live)
    }
}

impl fmt::Display for Liveness<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let said = match (self.reach, self.usage) {
            (Reach::Unreached, Usage::Unconsumed) => "no edge reaches it from a boundary entry and no edge takes it",
            (Reach::Unreached, Usage::Consumed) => "no edge reaches it from a boundary entry",
            (Reach::Reached, Usage::Unconsumed | Usage::Consumed) => "no edge takes it",
        };
        write!(formatter, "{}: {said}", self.node)
    }
}

impl lint_layout::Finding for Liveness<'_, '_> {
    fn fix(&self) -> Fix {
        match (self.reach, self.usage) {
            (Reach::Unreached, _) => Fix::new(
                "build it through an edge, a constructor or conversion from what the boundary hands in, or remove it",
            ),
            (Reach::Reached, _) => Fix::new("consume it through an edge or a port, or remove it"),
        }
    }
}

#[derive(Debug)]
pub(super) struct Tally {
    nodes: Count,
    unreached: Count,
    unconsumed: Count,
}

impl Tally {
    pub(super) fn counted(nodes: Count, found: &[Liveness<'_, '_>]) -> Tally {
        Tally {
            nodes,
            unreached: Count::new(found.iter().filter(|live| live.reach == Reach::Unreached).count()),
            unconsumed: Count::new(found.iter().filter(|live| live.usage == Usage::Unconsumed).count()),
        }
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "workspace: {} domain types, {} not reached from a boundary entry, {} taken by no edge",
            self.nodes, self.unreached, self.unconsumed
        )
    }
}

impl lint_layout::Finding for Tally {
    fn fix(&self) -> Fix {
        Fix::new("reach each type through an edge from the boundary and consume it through an edge or a port")
    }
}
