mod triangle;

use std::collections::BTreeSet;

use lint_core::{Headline, Lint, Report, SourceFile, Violation};
use transition_facts::Transitions;
use type_facts::Facts;

use crate::GraphLint;
use crate::end::{Hop, Node, Tuple};
use crate::graph::{Consuming, Graph, Hyperedge, Moves};
use crate::summary::Summary;
use triangle::{Detour, Tally, Triangle};

#[derive(Debug)]
pub struct CommutingTriangles;

impl GraphLint for CommutingTriangles {
    fn report_graph(facts: &Facts, transitions: &Transitions) -> Report {
        let graph = Graph::built(facts, transitions);
        let consuming = Consuming::indexed(&graph, Moves::All);
        let triangles: Vec<Triangle<'_, '_>> = graph
            .hyperedges()
            .iter()
            .filter_map(Hyperedge::written)
            .filter_map(|direct| Triangle::closing(direct, Self::paths(direct, &consuming)))
            .collect();
        Report::new(
            Headline::new(
                "report only, commuting triangles over the transition hypergraph, where an edge runs from the set of its domain inputs to its domain output and primitives, black boxes and interface holes are always at hand: a written edge from inputs S to C beside a path of two written edges, the first from part of S to a new type B, the second taking B and otherwise only types of S to C; either the direct edge is redundant and routes through B, or the two must agree and the equation is a law to declare; up to three paths are shown",
            ),
            Summary::placed(graph.nodes().values().copied().min(), Tally::counted(&triangles))
                .into_iter()
                .chain(triangles.iter().map(Violation::from))
                .collect(),
        )
    }
}

impl CommutingTriangles {
    fn paths<'graph, 'found>(
        direct: &'graph Hyperedge<'found>,
        consuming: &Consuming<'graph, 'found>,
    ) -> Vec<Detour<'graph, 'found>> {
        let hop: &Hop = direct.as_ref();
        let end = hop.reached();
        let tuple: &Tuple = hop.as_ref();
        let start: BTreeSet<&Node> = tuple.iter().collect();
        tuple
            .iter()
            .flat_map(|node| consuming.led(node))
            .filter_map(Hyperedge::written)
            .filter_map(|first| first.other(direct))
            .filter(|first| AsRef::<Tuple>::as_ref(AsRef::<Hop>::as_ref(*first)).within(&start).is_some())
            .filter(|first| !start.contains(AsRef::<Hop>::as_ref(*first).reached()))
            .flat_map(|first| {
                let middle = AsRef::<Hop>::as_ref(first).reached();
                let available: BTreeSet<&Node> = start.iter().copied().chain([middle]).collect();
                consuming
                    .taking(middle)
                    .filter_map(Hyperedge::written)
                    .filter_map(|second| second.other(direct))
                    .filter(|second| AsRef::<Hop>::as_ref(*second).reached() == end)
                    .filter(move |second| AsRef::<Hop>::as_ref(*second).fired(&available).is_some())
                    .map(move |second| Detour::new(first, second))
                    .collect::<Vec<Detour<'graph, 'found>>>()
            })
            .collect()
    }
}

impl Lint for CommutingTriangles {
    fn report(files: &[SourceFile]) -> Report {
        Self::report_graph(&Facts::of_files(files), &Transitions::of_files(files))
    }
}
