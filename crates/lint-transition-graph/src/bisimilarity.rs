mod behaviour;
mod group;

use std::collections::BTreeMap;

use lint_core::{Headline, Lint, Report, SourceFile, Violation};
use transition_facts::Transitions;
use type_facts::Facts;

use crate::GraphLint;
use crate::count::Count;
use crate::graph::Graph;
use crate::summary::Summary;
use behaviour::Behaviour;
use group::{Group, Tally};

#[derive(Debug)]
pub struct Bisimilarity;

impl GraphLint for Bisimilarity {
    fn report_graph<'file>(facts: &Facts<'file>, transitions: &Transitions<'file>) -> Report<'file> {
        let graph = Graph::built(facts, transitions);
        let behaviour = Behaviour::read(&graph);
        let blocks = behaviour.refined();
        let grouped = blocks.iter().fold(BTreeMap::new(), |mut groups: BTreeMap<_, Vec<_>>, (node, block)| {
            groups.entry(*block).or_default().push(*node);
            groups
        });
        let groups = Group::ranked(grouped.into_values(), &graph, &behaviour);
        Report::new(
            Headline::new(
                "report only, bisimilar types: partition refinement over the domain types on their moves beyond any trait, each labelled written for inherent and free functions, lift for a derived lift and the operation for a store operation, carrying the other domain inputs of a multi-input edge, and leading to an equivalent type or to the same black box; trait impls are the shared interface the fillers of a trait already group under (D7), so they are not evidence; types in one group answer the same labelled moves into the same groups, so they behave identically; groups whose types make fewer than two such moves are left out; candidates to merge into one type or one generic",
            ),
            Summary::placed(
                graph.nodes().values().copied().min(),
                Tally::counted(Count::new(graph.nodes().len()), &behaviour, &groups),
            )
            .into_iter()
            .chain(groups.iter().map(Violation::from))
            .collect(),
        )
    }
}

impl Lint for Bisimilarity {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Self::report_graph(&Facts::of_files(files), &Transitions::of_files(files))
    }
}
