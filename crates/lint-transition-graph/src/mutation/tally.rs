use std::fmt;
use std::iter;

use crate::transitions::Ground;
use graph::Mutation;
use lint_core::Fix;
use lint_core::{Site, Violation};
use lint_layout::Name;
use newtype::Counter;

use crate::count::Count;
use crate::edge::Edge;
use crate::forms::{Lifted, Stored, Written};

#[derive(Debug, Default)]
pub(super) struct Counted<'edge, 'found, 'file>(Vec<&'edge Edge<'found, 'file>>);

impl<'edge, 'found, 'file> FromIterator<&'edge Edge<'found, 'file>> for Counted<'edge, 'found, 'file> {
    fn from_iter<Edges: IntoIterator<Item = &'edge Edge<'found, 'file>>>(edges: Edges) -> Self {
        Counted(
            edges
                .into_iter()
                .filter_map(|edge| {
                    let stored = iter::once(edge).collect::<Stored<'edge, 'found, 'file>>().into_iter().next();
                    stored.or_else(|| Tally::mutating(edge).and(Some(edge)))
                })
                .collect(),
        )
    }
}

impl<'edge, 'found, 'file> IntoIterator for Counted<'edge, 'found, 'file> {
    type Item = &'edge Edge<'found, 'file>;
    type IntoIter = std::vec::IntoIter<&'edge Edge<'found, 'file>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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
    fn mutating(edge: &Edge<'_, '_>) -> Option<Mutation> {
        (*AsRef::<Ground>::as_ref(edge.transition()) == Ground::Core)
            .then_some(edge.transition().function().mutation())
            .filter(|mutation| matches!(mutation, Mutation::ThroughSelf | Mutation::ThroughParameter))
    }

    pub(super) fn counted<'edge, 'found: 'edge, 'file: 'found>(
        edges: impl Iterator<Item = &'edge Edge<'found, 'file>>,
    ) -> Tally {
        let all: Vec<&'edge Edge<'found, 'file>> = edges.collect();
        let stored = all.iter().copied().collect::<Stored<'edge, 'found, 'file>>().into_iter().count();
        let written = all.iter().copied().collect::<Written<'edge, 'found, 'file>>();
        let lifts = all.iter().copied().collect::<Lifted<'edge, 'found, 'file>>();
        written
            .into_iter()
            .chain(lifts)
            .fold(Tally { operations: Count::new(stored), ..Tally::default() }, |tally, edge| {
                tally.by_mutation(edge.transition().function().mutation()).fixed(edge.fixed_by())
            })
    }

    fn by_mutation(self, mutation: Mutation) -> Tally {
        match mutation {
            Mutation::ThroughSelf => Tally { through_self: self.through_self.incremented(), ..self },
            Mutation::ThroughParameter | Mutation::Absent => {
                Tally { through_parameter: self.through_parameter.incremented(), ..self }
            }
        }
    }

    fn fixed(self, fixed_by: Option<&Name>) -> Tally {
        fixed_by.map_or(self, |_| Tally { fixed: self.fixed.incremented(), ..self })
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
