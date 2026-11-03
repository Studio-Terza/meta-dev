mod bisimilarity;
mod count;
mod coverage;
mod crate_transitions;
mod duplicates;
mod edge;
mod end;
mod graph;
mod graph_lint;
mod hidden_states;
mod live_nodes;
mod motifs;
mod mutation;
mod summary;
mod tangles;
mod triangles;
mod workspace_types;

use std::collections::BTreeMap;

use lint_core::{Headline, Lint, Report, Site, SourceFile};
use lint_placement::CrateName;
use transition_facts::Transitions;
use type_facts::Facts;

use coverage::Coverage;
use crate_transitions::CrateTransitions;
use edge::Edge;
use workspace_types::WorkspaceTypes;

pub use bisimilarity::Bisimilarity;
pub use duplicates::{DuplicateTransitions, OneEdge};
pub use graph_lint::GraphLint;
pub use hidden_states::HiddenStates;
pub use live_nodes::LiveNodes;
pub use motifs::RepeatedMotifs;
pub use mutation::CoreMutation;
pub use tangles::Tangles;
pub use triangles::CommutingTriangles;

#[derive(Debug)]
pub struct TransitionGraph;

impl GraphLint for TransitionGraph {
    fn report_graph(facts: &Facts, transitions: &Transitions) -> Report {
        let types = WorkspaceTypes::declared(facts, transitions);
        let edges: Vec<Edge<'_>> = transitions.all().iter().map(|found| types.edge(found)).collect();
        let grouped = edges.iter().fold(BTreeMap::new(), |mut crates: BTreeMap<&CrateName, Vec<&Edge<'_>>>, edge| {
            crates.entry(edge.transition().as_ref()).or_default().push(edge);
            crates
        });
        Report::new(
            Headline::new(
                "report only, the transition graph per crate: nodes are domain types, interface nodes (traits, the Fn traits included, for generic, impl Trait and dyn parameters) and external black boxes; written edges are bodies with a signature, trait impls included, from their input types to their output type, and a domain pair joins the set of an edge's domain inputs to its domain output, so an edge with several domain inputs is one hyperedge; derived lifts (map through a container, field focus, match routing over a sum) and store operations (&mut self methods in a store module) are counted apart; coverage is the share of written edges with no bare primitive among their inputs and output, a primitive a foreign trait fixes reading as that trait's interface node; raise it with a newtype, an enum or a standard trait",
            ),
            grouped
                .iter()
                .filter_map(|(name, found)| CrateTransitions::measured(&types, name, found).violation())
                .chain(
                    Coverage::tallied(edges.iter())
                        .violation(edges.iter().map(|edge| AsRef::<Site>::as_ref(edge.transition())).min()),
                )
                .collect(),
        )
    }
}

impl Lint for TransitionGraph {
    fn report(files: &[SourceFile]) -> Report {
        Self::report_graph(&Facts::of_files(files), &Transitions::of_files(files))
    }
}
