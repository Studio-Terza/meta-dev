use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Fix;
use lint_placement::CrateName;

use super::tally::Tally;
use crate::edge::{Edge, Role};

#[derive(Debug)]
pub(super) struct CrateMutation<'found> {
    name: &'found CrateName,
    first: Option<&'found Site>,
    tally: Tally,
    operations: Vec<&'found Edge<'found>>,
}

impl<'found> CrateMutation<'found> {
    pub(super) fn measured(name: &'found CrateName, edges: &[&'found Edge<'found>]) -> CrateMutation<'found> {
        CrateMutation {
            name,
            first: edges.iter().map(|edge| AsRef::<Site>::as_ref(edge.transition())).min(),
            tally: Tally::counted(edges.iter().copied()),
            operations: edges.iter().copied().filter(|edge| edge.role() == Role::Operation).collect(),
        }
    }

    pub(super) fn reported(self) -> Option<Violation> {
        self.first.map(|site| Violation::new(site.clone(), &self))
    }
}

impl fmt::Display for CrateMutation<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "crate {}: {}", self.name, self.tally)?;
        self.operations.iter().try_for_each(|operation| {
            write!(formatter, ", {} applies {}", operation.transition(), operation.ends().applied())
        })
    }
}

impl lint_layout::Finding for CrateMutation<'_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "move each mutation into a store operation, a &mut self method in a store module, or return a new value",
        )
    }
}
