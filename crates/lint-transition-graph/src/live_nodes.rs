mod liveness;

use std::collections::BTreeSet;

use lint_core::{Headline, Lint, Report, SourceFile, Violation};
use transition_facts::{Ground, Transitions};
use type_facts::Facts;

use crate::GraphLint;
use crate::count::Count;
use crate::end::Node;
use crate::graph::{Consuming, Graph, Moves};
use crate::summary::Summary;
use liveness::{Liveness, Reach, Tally, Usage};

#[derive(Debug)]
pub struct LiveNodes;

impl GraphLint for LiveNodes {
    fn report_graph(facts: &Facts, transitions: &Transitions) -> Report {
        let graph = Graph::built(facts, transitions);
        let entered: BTreeSet<&Node> = graph
            .edges()
            .iter()
            .filter_map(|edge| edge.ends().entered(*AsRef::<Ground>::as_ref(edge.transition())))
            .collect();
        let reached = Consuming::indexed(&graph, Moves::All).closure(entered);
        let consumed: BTreeSet<&Node> = graph.edges().iter().flat_map(|edge| edge.ends().taken()).collect();
        let found: Vec<Liveness<'_>> = graph
            .nodes()
            .iter()
            .filter_map(|(node, site)| {
                let reach = reached.get(node).map_or(Reach::Unreached, |_| Reach::Reached);
                let usage = consumed.get(node).map_or(Usage::Unconsumed, |_| Usage::Consumed);
                Liveness::judged(node, site, reach, usage)
            })
            .collect();
        Report::new(
            Headline::new(
                "report only, live nodes: every domain type is reachable from a boundary entry (an edge in a ports or boundary module, or one taking no domain type) along written edges, lifts and store operations, an edge with several domain inputs firing only once all of them are reached, and is consumed by some edge; a type no edge reaches is built only inside bodies, a type no edge takes is a dead end",
            ),
            Summary::placed(
                graph.nodes().values().copied().min(),
                Tally::counted(Count::new(graph.nodes().len()), &found),
            )
            .into_iter()
            .chain(found.iter().map(Violation::from))
            .collect(),
        )
    }
}

impl Lint for LiveNodes {
    fn report(files: &[SourceFile]) -> Report {
        Self::report_graph(&Facts::of_files(files), &Transitions::of_files(files))
    }
}
