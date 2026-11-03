use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Fix;
use type_facts::Transition;

use super::family::Family;
use crate::count::Count;
use crate::end::{Class, Ends};

#[derive(Debug)]
pub(super) struct Comparison {
    compared: Count,
    families: Count,
    repeated: Count,
    primitive: Count,
    foreign: Count,
}

impl Comparison {
    pub(super) fn of_families(ends: &[Ends], families: &[Family<'_>]) -> Comparison {
        let classed = |class: Class| Count::new(ends.iter().filter(|found| found.class() == class).count());
        Comparison {
            compared: Count::new(ends.iter().filter(|found| found.key().is_some()).count()),
            families: Count::new(families.len()),
            repeated: Count::new(families.iter().map(Family::size).sum()),
            primitive: classed(Class::Primitive),
            foreign: classed(Class::Foreign),
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
            "workspace: {} transitions between workspace types compared, {} input-output pairs joined more than once by {} transitions; not compared: {} touching a primitive or unit, {} touching a foreign or generic type",
            self.compared, self.families, self.repeated, self.primitive, self.foreign
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
