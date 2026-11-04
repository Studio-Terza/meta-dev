mod algebra;
mod algebra_duplicates;
mod algebra_laws;
mod class;
mod containment;
mod count;
mod coverage;
mod crate_transitions;
mod duplicates;
mod edge;
mod end;
mod form;
mod graph_lint;
mod key;
mod listed;
mod mutation;
mod system;
mod workspace_types;

use lint_core::{Headline, Report};

use coverage::Coverage;
use crate_transitions::CrateTransitions;
use edge::Edge;

pub use algebra_duplicates::AlgebraDuplicates;
pub use algebra_laws::AlgebraLaws;
pub use containment::{AcyclicQuotients, ConnectedClusters, InterfaceByConsumers, LowestCluster};
pub use duplicates::{DuplicateTransitions, OneEdge};
pub use graph_lint::{Clustered, GraphLint};
pub use mutation::CoreMutation;
pub use system::{Extraction, TypeSystem};

#[derive(Debug)]
pub struct TransitionGraph;

impl GraphLint for TransitionGraph {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file> {
        let types = system.types();
        let edges = system.edges();
        let grouped = Edge::by_crate(edges.iter());
        Report::new(
            Headline::new(
                "report only, the transition graph per crate: nodes are domain types, interface nodes (traits, the Fn traits included, for generic, impl Trait and dyn parameters) and external black boxes; written edges are bodies with a signature, trait impls included, from their input types to their output type, and a domain pair joins the set of an edge's domain inputs to its domain output, so an edge with several domain inputs is one hyperedge; derived lifts (map through a container, field focus, match routing over a sum) and store operations (&mut self methods in a store module) are counted apart; of the written edges, 'of which N algebra operations, N partial' counts those whose value inputs and output are one type (acted on by any external parameter), partial when they return Option or Result of that type, a collection of the type staying a transition; coverage is the share of written edges with no bare primitive among their inputs and output, a primitive a foreign trait fixes reading as that trait's interface node; raise it with a newtype, an enum or a standard trait",
            ),
            grouped
                .iter()
                .filter_map(|(name, found)| {
                    CrateTransitions::measured(types, name, found, Coverage::tallied(system, found.iter().copied()))
                        .violation()
                })
                .chain(Coverage::tallied(system, edges.iter()).violation(edges.iter().map(Edge::site).min()))
                .collect(),
        )
    }
}
