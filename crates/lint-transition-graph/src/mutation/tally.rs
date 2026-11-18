use std::fmt;

use crate::transitions::Ground;
use graph::Mutation;
use lint_core::Fix;
use lint_core::{Site, Violation};
use lint_layout::Name;
use newtype::Counter;

use crate::count::Count;
use crate::edge::Edge;

#[derive(Clone, Copy, Debug)]
pub(super) struct Counted<'edge, 'found, 'file>(&'edge Edge<'found, 'file>);

impl<'edge, 'found, 'file> From<Counted<'edge, 'found, 'file>> for &'edge Edge<'found, 'file> {
    fn from(counted: Counted<'edge, 'found, 'file>) -> &'edge Edge<'found, 'file> {
        counted.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Tally {
    through_self: Count,
    through_parameter: Count,
    fixed: Count,
    operations: Count,
}

impl Tally {
    pub(super) fn counts<'edge, 'found, 'file>(
        edge: &'edge Edge<'found, 'file>,
    ) -> Option<Counted<'edge, 'found, 'file>> {
        let mutating = Some(edge)
            .filter(|candidate| *AsRef::<Ground>::as_ref(candidate.transition()) == Ground::Core)
            .filter(|core| {
                matches!(core.transition().function().mutation(), Mutation::ThroughSelf | Mutation::ThroughParameter)
            })
            .map(Counted);
        edge.form().storing(edge).map(|stored| Counted(stored.as_ref())).or(mutating)
    }

    pub(super) fn counted<'edge, 'found: 'edge, 'file: 'found>(
        edges: impl Iterator<Item = &'edge Edge<'found, 'file>>,
    ) -> Tally {
        edges.fold(Tally::default(), |tally, edge| {
            tally.by_form(edge, edge.transition().function().mutation()).fixed(edge, edge.fixed_by())
        })
    }

    fn by_form(self, edge: &Edge<'_, '_>, mutation: Mutation) -> Tally {
        edge.form()
            .storing(Tally { operations: self.operations.incremented(), ..self })
            .map_or_else(|| self.by_mutation(mutation), |stored| *stored.as_ref())
    }

    fn by_mutation(self, mutation: Mutation) -> Tally {
        match mutation {
            Mutation::ThroughSelf => Tally { through_self: self.through_self.incremented(), ..self },
            Mutation::ThroughParameter | Mutation::Absent => {
                Tally { through_parameter: self.through_parameter.incremented(), ..self }
            }
        }
    }

    fn fixed(self, edge: &Edge<'_, '_>, fixed_by: Option<&Name>) -> Tally {
        let fixing = fixed_by.map_or(self, |_| Tally { fixed: self.fixed.incremented(), ..self });
        edge.form().storing(self).map_or(fixing, |stored| *stored.as_ref())
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

impl lint_core::Finding for Workspace {
    fn fix(&self) -> Fix {
        Fix::new(
            "move each mutation into a store operation, a &mut self method in a store module, or return a new value",
        )
    }
}
