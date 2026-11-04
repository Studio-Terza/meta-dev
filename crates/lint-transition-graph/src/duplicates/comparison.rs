use std::fmt;

use lint_core::Fix;
use lint_core::{Site, Violation};
use transition_facts::Transitions;

use super::family::Family;
use crate::class::Class;
use crate::count::Count;
use crate::edge::Edge;
use newtype::Counter;

#[derive(Debug)]
pub(super) struct Comparison {
    compared: Count,
    families: Count,
    repeated: Count,
    primitive: Count,
    open: Count,
    apart: Count,
}

impl Comparison {
    pub(super) fn of_families(edges: &[Edge<'_, '_>], families: &[Family<'_, '_>]) -> Comparison {
        let written: Vec<&Edge<'_, '_>> = edges.iter().filter(|edge| edge.form().written().is_some()).collect();
        let compared = Count::new(written.iter().filter(|edge| edge.ends().key().is_some()).count());
        let primitive = Count::new(written.iter().filter(|edge| edge.ends().class() == Class::Primitive).count());
        Comparison {
            compared,
            families: Count::new(families.len()),
            repeated: Count::total(families.iter().map(Family::size)),
            primitive,
            open: Count::new(written.len().saturating_sub(compared.count()).saturating_sub(primitive.count())),
            apart: Count::new(edges.len().saturating_sub(written.len())),
        }
    }

    pub(super) fn violation<'file>(&self, transitions: &Transitions<'file>) -> Option<Violation<'file>> {
        transitions
            .all()
            .map(|found| *AsRef::<Site<'file>>::as_ref(&found))
            .min()
            .map(|first| Violation::new(first, self))
    }
}

impl fmt::Display for Comparison {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "workspace: {} written edges between domain types compared, {} input-output pairs joined more than once by {} edges; not compared: {} touching a bare primitive, {} touching an interface node or black box, {} derived lifts and store operations",
            self.compared, self.families, self.repeated, self.primitive, self.open, self.apart
        )
    }
}

impl lint_core::Finding for Comparison {
    fn fix(&self) -> Fix {
        Fix::new(
            "route each repeated input-output pair through one transition, as From, TryFrom or FromStr where it fits",
        )
    }
}
