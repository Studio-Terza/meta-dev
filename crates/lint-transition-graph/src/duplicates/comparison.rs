use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Fix;
use transition_facts::Transition;

use super::family::Family;
use crate::count::Count;
use crate::edge::{Edge, Role};
use crate::end::Class;
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
    pub(super) fn of_families(edges: &[Edge<'_>], families: &[Family<'_>]) -> Comparison {
        let written: Vec<&Edge<'_>> = edges.iter().filter(|edge| edge.role() == Role::Written).collect();
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

    pub(super) fn violation(&self, transitions: &[Transition]) -> Option<Violation> {
        transitions.iter().map(AsRef::<Site>::as_ref).min().map(|first| Violation::new(first.clone(), self))
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

impl lint_layout::Finding for Comparison {
    fn fix(&self) -> Fix {
        Fix::new(
            "route each repeated input-output pair through one transition, as From, TryFrom or FromStr where it fits",
        )
    }
}
