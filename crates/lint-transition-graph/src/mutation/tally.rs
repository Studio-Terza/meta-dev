use std::fmt;

use lint_core::Fix;
use lint_core::{Site, Violation};
use lint_layout::Name;
use newtype::Counter;
use transition_facts::{Ground, Mutation};

use crate::count::Count;
use crate::edge::Edge;
use crate::form::Form;

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
        edge.stored().or(mutating)
    }

    pub(super) fn counted<'edge, 'found: 'edge, 'file: 'found>(
        edges: impl Iterator<Item = &'edge Edge<'found, 'file>>,
    ) -> Self {
        edges.fold(Self::default(), |tally, edge| {
            tally.by_form(edge.form(), edge.transition().function().mutation()).fixed(edge.form(), edge.fixed_by())
        })
    }

    fn by_form(self, form: &Form<'_>, mutation: Mutation) -> Self {
        let mutating = self.by_mutation(mutation);
        form.sorted(mutating, mutating, Self { operations: self.operations.incremented(), ..self })
    }

    fn by_mutation(self, mutation: Mutation) -> Self {
        match mutation {
            Mutation::ThroughSelf => Self { through_self: self.through_self.incremented(), ..self },
            Mutation::ThroughParameter | Mutation::Absent => {
                Self { through_parameter: self.through_parameter.incremented(), ..self }
            }
        }
    }

    fn fixed(self, form: &Form<'_>, fixed_by: Option<&Name>) -> Self {
        let fixing = fixed_by.map_or(self, |_| Self { fixed: self.fixed.incremented(), ..self });
        form.sorted(fixing, fixing, self)
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
