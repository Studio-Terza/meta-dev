use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Fix;
use newtype::Counter;
use transition_facts::{Ground, Mutation};

use crate::count::Count;
use crate::edge::{Edge, Role};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Tally {
    through_self: Count,
    through_parameter: Count,
    fixed: Count,
    operations: Count,
}

impl Tally {
    pub(super) fn counts<'edge, 'found, 'file>(edge: &'edge Edge<'found, 'file>) -> Option<&'edge Edge<'found, 'file>> {
        match (edge.role(), *AsRef::<Ground>::as_ref(edge.transition()), edge.transition().mutation()) {
            (Role::Operation, _, _) | (_, Ground::Core, Mutation::ThroughSelf | Mutation::ThroughParameter) => {
                Some(edge)
            }
            _ => None,
        }
    }

    pub(super) fn counted<'edge, 'found: 'edge, 'file: 'found>(
        edges: impl Iterator<Item = &'edge Edge<'found, 'file>>,
    ) -> Tally {
        edges.fold(Tally::default(), |tally, edge| {
            let counted = match (edge.role(), edge.transition().mutation()) {
                (Role::Operation, _) => Tally { operations: tally.operations.incremented(), ..tally },
                (_, Mutation::ThroughSelf) => Tally { through_self: tally.through_self.incremented(), ..tally },
                (_, Mutation::ThroughParameter | Mutation::Absent) => {
                    Tally { through_parameter: tally.through_parameter.incremented(), ..tally }
                }
            };
            match (edge.role(), edge.fixed_by()) {
                (Role::Written | Role::Lift, Some(_)) => Tally { fixed: counted.fixed.incremented(), ..counted },
                _ => counted,
            }
        })
    }

    pub(super) fn violation(self, first: Option<Site<'_>>) -> Option<Violation<'_>> {
        first.map(|site| Violation::new(site, Workspace(self)))
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mutating = self.through_self.plus(self.through_parameter);
        write!(
            formatter,
            "{} core edges take &mut, {} through &mut self, {} through a &mut parameter only, {} with a signature fixed by a foreign trait, {} to migrate; {} store operations",
            mutating,
            self.through_self,
            self.through_parameter,
            self.fixed,
            Count::new(mutating.count().saturating_sub(self.fixed.count())),
            self.operations
        )
    }
}

#[derive(Debug)]
struct Workspace(Tally);

impl fmt::Display for Workspace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "workspace: {}", self.0)
    }
}

impl lint_layout::Finding for Workspace {
    fn fix(&self) -> Fix {
        Fix::new(
            "move each mutation into a store operation, a &mut self method in a store module, or return a new value",
        )
    }
}
