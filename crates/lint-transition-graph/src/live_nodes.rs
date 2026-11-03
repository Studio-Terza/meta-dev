mod liveness;

use std::collections::BTreeSet;
use std::iter;

use lint_core::{Headline, Lint, Report, SourceFile, Violation};
use transition_facts::{Ground, Transitions};
use type_facts::Facts;

use crate::GraphLint;
use crate::count::Count;
use crate::end::{Node, Step};
use crate::graph::Graph;
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
        let leaving = graph.leaving();
        let reached = Self::reached(entered, |node| {
            leaving.get(node).into_iter().flatten().map(|arc| AsRef::<Step>::as_ref(*arc).reached()).collect()
        });
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
                "report only, live nodes: every domain type is reachable from a boundary entry (an edge in a ports or boundary module, or one taking no domain type) along written edges and lifts, and is consumed by some edge; a type no edge reaches is built only inside bodies, a type no edge takes is a dead end",
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

impl LiveNodes {
    fn reached<'node>(
        entered: BTreeSet<&'node Node>,
        next: impl Fn(&'node Node) -> Vec<&'node Node>,
    ) -> BTreeSet<&'node Node> {
        let mut seen = entered.clone();
        iter::successors(Some(entered), |frontier: &BTreeSet<&'node Node>| {
            let fresh: BTreeSet<&'node Node> =
                frontier.iter().flat_map(|node| next(node)).filter(|reached| seen.insert(*reached)).collect();
            fresh.first().map(|_| fresh.clone())
        })
        .flatten()
        .collect()
    }
}

impl Lint for LiveNodes {
    fn report(files: &[SourceFile]) -> Report {
        Self::report_graph(&Facts::of_files(files), &Transitions::of_files(files))
    }
}
