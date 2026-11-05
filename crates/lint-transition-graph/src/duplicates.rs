mod comparison;
mod family;

use lint_core::{Headline, Report};

use crate::system::TypeSystem;
use crate::{Covered, GraphLint};
use comparison::Comparison;
use family::Family;

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

#[derive(Debug)]
pub struct OneEdge;

impl GraphLint for OneEdge {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file> {
        let edges = system.edges();
        Report::new(
            Headline::new(
                "one edge per pair: in shipped code, a set of input types reaches one output type by one written edge (function, method or trait impl), read through references, Option, Result and collections, compared where every type is a domain type; a trait impl also takes its implementing type when no input or output carries it; operations whose inputs and output are all one type (A, A -> A) are that type's algebra, judged by its laws, not edges; derived lifts and store operations are not written edges",
            ),
            Family::repeated_among(edges).iter().filter_map(Family::violation).collect(),
        )
    }
}

impl Covered for OneEdge {
    type Coverage = DuplicateTransitions;
}
