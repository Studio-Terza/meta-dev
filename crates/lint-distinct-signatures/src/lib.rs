use std::fmt;

use equivalence::{Equivalence, Erased};
use lint_core::{Fix, Headline, Report, Violation};
use lint_transition_graph::{Alike, GraphLint, TypeSystem};

#[derive(Debug)]
pub struct DistinctSignatures;

impl DistinctSignatures {
    const EQUIVALENCE: Equivalence = Equivalence::erasing(&[Erased::Names]);
}

impl GraphLint for DistinctSignatures {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file> {
        Report::new(
            Headline::new(
                "two methods of one type with the same signature hide a missing type or type parameter: every inherent method of a type, across all files of its crate, is compared on the type graph by its inputs (the receiver first), its output, whether each is optional, fallible or many, and whether it mutates self or a parameter; trait impls are fixed by their trait and are not compared",
            ),
            Alike::grouped(system, Self::EQUIVALENCE).iter().filter_map(Shared::violation).collect(),
        )
    }
}

struct Shared<'alike, 'system, 'found, 'file>(&'alike Alike<'system, 'found, 'file>);

impl<'file> Shared<'_, '_, '_, 'file> {
    fn violation(alike: &Alike<'_, '_, 'file>) -> Option<Violation<'file>> {
        alike.site().map(|site| Violation::new(site, Shared(alike)))
    }
}

impl fmt::Display for Shared<'_, '_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl lint_core::Finding for Shared<'_, '_, '_, '_> {
    fn fix(&self) -> Fix {
        Fix::new("distinguish the types they take or return, or keep one method and route the other through it")
    }
}
