use std::fmt;

use lint_core::Fix;
use lint_core::{Site, Violation};
use lint_placement::CrateName;

use super::tally::Tally;
use crate::edge::Edge;

#[derive(Debug)]
pub(super) struct CrateMutation<'found, 'file> {
    name: &'found CrateName,
    first: Option<Site<'file>>,
    tally: Tally,
    operations: Vec<&'found Edge<'found, 'file>>,
}

impl<'found, 'file> CrateMutation<'found, 'file> {
    pub(super) fn measured(name: &'found CrateName, edges: &[&'found Edge<'found, 'file>]) -> Self {
        CrateMutation {
            name,
            first: edges.iter().map(|edge| edge.site()).min(),
            tally: Tally::counted(edges.iter().copied()),
            operations: edges.iter().copied().filter_map(Edge::stored).collect(),
        }
    }

    pub(super) fn reported(self) -> Option<Violation<'file>> {
        self.first.map(|site| Violation::new(site, &self))
    }
}

impl fmt::Display for CrateMutation<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "crate {}: {}", self.name, self.tally)?;
        self.operations.iter().try_for_each(|operation| {
            write!(formatter, ", {} applies {}", operation.transition(), operation.ends().applied())
        })
    }
}

impl lint_core::Finding for CrateMutation<'_, '_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "move each mutation into a store operation, a &mut self method in a store module, or return a new value",
        )
    }
}
