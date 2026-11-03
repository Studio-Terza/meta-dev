mod triangle;

use std::collections::BTreeMap;

use lint_core::{Headline, Lint, Report, SourceFile, Violation};
use transition_facts::Transitions;
use type_facts::Facts;

use crate::GraphLint;
use crate::end::{Node, Step};
use crate::graph::{Arc, Graph};
use crate::summary::Summary;
use triangle::{Detour, Tally, Triangle};

#[derive(Debug)]
pub struct CommutingTriangles;

impl GraphLint for CommutingTriangles {
    fn report_graph(facts: &Facts, transitions: &Transitions) -> Report {
        let graph = Graph::built(facts, transitions);
        let leaving = graph.leaving();
        let triangles: Vec<Triangle<'_, '_>> = graph
            .arcs()
            .iter()
            .filter_map(Arc::written)
            .filter_map(|arc| Triangle::closing(arc, Self::paths(arc, &leaving)))
            .collect();
        Report::new(
            Headline::new(
                "report only, commuting triangles: a written edge from A to C beside a path of written edges A to B to C between domain types; either the direct edge is redundant and routes through B, or the two must agree and the equation is a law to declare; up to three paths are shown",
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
        direct: &Arc<'found>,
        leaving: &BTreeMap<&'graph Node, Vec<&'graph Arc<'found>>>,
    ) -> Vec<Detour<'graph, 'found>> {
        let step: &Step = direct.as_ref();
        let start: &Node = step.as_ref();
        let end = step.reached();
        leaving
            .get(start)
            .into_iter()
            .flatten()
            .copied()
            .filter_map(Arc::written)
            .filter(|first| {
                let middle = AsRef::<Step>::as_ref(*first).reached();
                middle != start && middle != end
            })
            .flat_map(|first| {
                let middle = AsRef::<Step>::as_ref(first).reached();
                leaving
                    .get(middle)
                    .into_iter()
                    .flatten()
                    .copied()
                    .filter_map(Arc::written)
                    .filter(move |second| AsRef::<Step>::as_ref(*second).reached() == end)
                    .map(move |second| Detour::new(first, second))
            })
            .collect()
    }
}

impl Lint for CommutingTriangles {
    fn report(files: &[SourceFile]) -> Report {
        Self::report_graph(&Facts::of_files(files), &Transitions::of_files(files))
    }
}
