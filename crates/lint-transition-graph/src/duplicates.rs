mod comparison;
mod family;

use lint_core::{Headline, Lint, Report, SourceFile};
use transition_facts::Transitions;
use type_facts::Facts;

use crate::GraphLint;
use crate::edge::Edge;
use crate::workspace_types::WorkspaceTypes;
use comparison::Comparison;
use family::Family;

#[derive(Debug)]
pub struct DuplicateTransitions;

impl GraphLint for DuplicateTransitions {
    fn report_graph<'file>(facts: &Facts<'file>, transitions: &Transitions<'file>) -> Report<'file> {
        let types = WorkspaceTypes::declared(facts, transitions);
        let edges: Vec<Edge<'_, 'file>> = transitions.all().map(|found| types.edge(found)).collect();
        let families = Family::repeated_among(&edges);
        Report::new(
            Headline::new(
                "report only, duplicate transitions, the coverage of one_edge: written edges between domain types compared, the pairs joined more than once (one_edge lists them), and the edges left out of the comparison",
            ),
            Comparison::of_families(&edges, &families).violation(transitions).into_iter().collect(),
        )
    }
}

impl Lint for DuplicateTransitions {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Self::report_graph(&Facts::of_files(files), &Transitions::of_files(files))
    }
}

#[derive(Debug)]
pub struct OneEdge;

impl GraphLint for OneEdge {
    fn report_graph<'file>(facts: &Facts<'file>, transitions: &Transitions<'file>) -> Report<'file> {
        let types = WorkspaceTypes::declared(facts, transitions);
        let edges: Vec<Edge<'_, 'file>> = transitions.all().map(|found| types.edge(found)).collect();
        Report::new(
            Headline::new(
                "one edge per pair: in shipped code, a set of input types reaches one output type by one written edge (function, method or trait impl), read through references, Option, Result and collections, compared where every type is a domain type; a trait impl also takes its implementing type when no input or output carries it; operations whose inputs and output are all one type (A, A -> A) are that type's algebra, judged by its laws, not edges; derived lifts and store operations are not written edges",
            ),
            Family::repeated_among(&edges).iter().filter_map(Family::violation).collect(),
        )
    }
}

impl Lint for OneEdge {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Self::report_graph(&Facts::of_files(files), &Transitions::of_files(files))
    }
}
