mod motif;

use std::collections::BTreeMap;

use lint_core::{Headline, Lint, Report, SourceFile, Violation};
use lint_layout::Name;
use transition_facts::Transitions;
use type_facts::Facts;

use crate::GraphLint;
use crate::end::Node;
use crate::graph::{Arc, Graph};
use crate::summary::Summary;
use motif::{Motif, Occurrence, Tally};

#[derive(Debug)]
pub struct RepeatedMotifs;

impl GraphLint for RepeatedMotifs {
    fn report_graph(facts: &Facts, transitions: &Transitions) -> Report {
        let graph = Graph::built(facts, transitions);
        let leaving = graph.leaving();
        let pairs: Vec<Occurrence<'_, '_>> = graph
            .arcs()
            .iter()
            .filter_map(Arc::written)
            .map(Occurrence::started)
            .flat_map(|path| Self::next(&path, &leaving))
            .collect();
        let triples: Vec<Occurrence<'_, '_>> = pairs.iter().flat_map(|path| Self::next(path, &leaving)).collect();
        let shaped = pairs.into_iter().chain(triples).fold(
            BTreeMap::new(),
            |mut shapes: BTreeMap<Vec<&Name>, Vec<Occurrence<'_, '_>>>, path| {
                shapes.entry(path.shape()).or_default().push(path);
                shapes
            },
        );
        let motifs =
            Motif::ranked(shaped.into_iter().filter_map(|(shape, found)| Motif::repeated(shape, found)).collect());
        Report::new(
            Headline::new(
                "report only, repeated motifs: the same chain of two or three written edges, named alike, over disjoint sets of domain types; each repeat is the same small transition shape written again, a candidate for a trait or a generic over the types; longest and most repeated first",
            ),
            Summary::placed(graph.nodes().values().copied().min(), Tally::counted(&motifs))
                .into_iter()
                .chain(motifs.iter().map(Violation::from))
                .collect(),
        )
    }
}

impl RepeatedMotifs {
    fn next<'graph, 'found>(
        path: &Occurrence<'graph, 'found>,
        leaving: &BTreeMap<&'graph Node, Vec<&'graph Arc<'found>>>,
    ) -> Vec<Occurrence<'graph, 'found>> {
        path.last()
            .and_then(|node| leaving.get(node))
            .into_iter()
            .flatten()
            .copied()
            .filter_map(Arc::written)
            .filter_map(|arc| path.extended(arc))
            .collect()
    }
}

impl Lint for RepeatedMotifs {
    fn report(files: &[SourceFile]) -> Report {
        Self::report_graph(&Facts::of_files(files), &Transitions::of_files(files))
    }
}
