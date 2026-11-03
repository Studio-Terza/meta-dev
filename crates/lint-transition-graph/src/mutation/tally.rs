use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::{Fix, Name};
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
        let mutating =
            Some(edge).filter(|edge| *AsRef::<Ground>::as_ref(edge.transition()) == Ground::Core).filter(|edge| {
                matches!(edge.transition().function().mutation(), Mutation::ThroughSelf | Mutation::ThroughParameter)
            });
        Some(edge).filter(|edge| edge.role() == Role::Operation).or(mutating)
    }

    pub(super) fn counted<'edge, 'found: 'edge, 'file: 'found>(
        edges: impl Iterator<Item = &'edge Edge<'found, 'file>>,
    ) -> Tally {
        edges.fold(Tally::default(), |tally, edge| {
            tally.by_role(edge.role(), edge.transition().function().mutation()).fixed(edge.role(), edge.fixed_by())
        })
    }

    fn by_role(self, role: Role, mutation: Mutation) -> Tally {
        match role {
            Role::Operation => Tally { operations: self.operations.incremented(), ..self },
            Role::Written | Role::Lift => self.by_mutation(mutation),
        }
    }

    fn by_mutation(self, mutation: Mutation) -> Tally {
        match mutation {
            Mutation::ThroughSelf => Tally { through_self: self.through_self.incremented(), ..self },
            Mutation::ThroughParameter | Mutation::Absent => {
                Tally { through_parameter: self.through_parameter.incremented(), ..self }
            }
        }
    }

    fn fixed(self, role: Role, fixed_by: Option<&Name>) -> Tally {
        match role {
            Role::Operation => self,
            Role::Written | Role::Lift => fixed_by.map_or(self, |_| Tally { fixed: self.fixed.incremented(), ..self }),
        }
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
