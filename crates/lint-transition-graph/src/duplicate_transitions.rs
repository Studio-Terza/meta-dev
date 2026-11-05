mod comparison;

use lint_core::{Headline, Report};

use crate::GraphLint;
use crate::family::Family;
use crate::system::TypeSystem;
use comparison::Comparison;

#[derive(Debug)]
pub struct DuplicateTransitions;

impl GraphLint for DuplicateTransitions {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file> {
        let edges = system.edges();
        let families = Family::repeated_among(edges);
        Report::new(
            Headline::new(
                "report only, duplicate transitions, the coverage of one_edge: written edges between domain types compared, the pairs joined more than once (one_edge lists them), and the edges left out of the comparison",
            ),
            Comparison::of_families(edges, &families).violation(system.transitions()).into_iter().collect(),
        )
    }
}
