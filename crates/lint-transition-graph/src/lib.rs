mod class;
mod containment;
mod count;
mod coverage;
mod crate_transitions;
mod duplicates;
mod edge;
mod end;
mod graph_lint;
mod key;
mod mutation;
mod workspace_types;

use lint_core::{Headline, Report};
use transition_facts::Transitions;
use type_facts::Facts;

use coverage::Coverage;
use crate_transitions::CrateTransitions;
use edge::Edge;
use workspace_types::WorkspaceTypes;

pub use containment::{
    AcyclicQuotients, ConnectedClusters, ConnectedDirectories, Containment, CrateInterfaces, InterfaceByConsumers,
    LowestCluster,
};
pub use duplicates::{DuplicateTransitions, OneEdge};
pub use graph_lint::{GraphLint, OnGraph};
pub use mutation::CoreMutation;

#[derive(Debug)]
pub struct TransitionGraph;

impl GraphLint for TransitionGraph {
    fn report_graph<'file>(facts: &Facts<'file>, transitions: &Transitions<'file>) -> Report<'file> {
        let types = WorkspaceTypes::declared(facts, transitions);
        let edges: Vec<Edge<'_, 'file>> = transitions.all().map(|found| types.edge(found)).collect();
        let grouped = Edge::by_crate(edges.iter());
        Report::new(
            Headline::new(
                "report only, the transition graph per crate: nodes are domain types, interface nodes (traits, the Fn traits included, for generic, impl Trait and dyn parameters) and external black boxes; written edges are bodies with a signature, trait impls included, from their input types to their output type, and a domain pair joins the set of an edge's domain inputs to its domain output, so an edge with several domain inputs is one hyperedge; derived lifts (map through a container, field focus, match routing over a sum) and store operations (&mut self methods in a store module) are counted apart; coverage is the share of written edges with no bare primitive among their inputs and output, a primitive a foreign trait fixes reading as that trait's interface node; raise it with a newtype, an enum or a standard trait",
            ),
            grouped
                .iter()
                .filter_map(|(name, found)| CrateTransitions::measured(&types, name, found).violation())
                .chain(Coverage::tallied(edges.iter()).violation(edges.iter().map(Edge::site).min()))
                .collect(),
        )
    }
}
