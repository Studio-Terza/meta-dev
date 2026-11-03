mod tangle;

use std::collections::{BTreeMap, BTreeSet};

use lint_core::{Headline, Lint, Report, SourceFile, Violation};
use transition_facts::Transitions;
use type_facts::Facts;

use crate::GraphLint;
use crate::end::Node;
use crate::graph::{Consuming, Graph, Moves};
use crate::summary::Summary;
use tangle::{Tally, Tangle};

#[derive(Debug)]
pub struct Tangles;

impl GraphLint for Tangles {
    fn report_graph<'file>(facts: &Facts<'file>, transitions: &Transitions<'file>) -> Report<'file> {
        let graph = Graph::built(facts, transitions);
        let consuming = Consuming::indexed(&graph, Moves::Unlifted);
        let reach: BTreeMap<&Node, BTreeSet<&Node>> = consuming
            .nodes()
            .map(|node| {
                let reached = consuming.closure(&BTreeSet::from([node]));
                (node, reached.into_iter().filter(|other| *other != node).collect())
            })
            .collect();
        let groups: BTreeSet<BTreeSet<&Node>> = reach
            .iter()
            .map(|(node, reached)| Self::component(node, reached, &reach))
            .filter(|group| group.len() > 1)
            .collect();
        let tangles =
            Tangle::ranked(groups.into_iter().map(|members| Tangle::closed(members, &consuming, &graph)).collect());
        Report::new(
            Headline::new(
                "report only, tangles: strongly connected groups of domain types along written edges, trait impls and store operations, over the transition hypergraph: a type reaches another when the other follows from it alone, each edge firing only once all its domain inputs are reached (primitives, black boxes and interface holes are always at hand); the core should be acyclic (D11), with cycles only as lifecycles declared through states! (whose types the macro generates, so they are not nodes here) or as round trips under a roundtrip! law; a group of two joined only by conversions is listed as a round trip",
            ),
            Summary::placed(graph.nodes().values().copied().min(), Tally::counted(&tangles))
                .into_iter()
                .chain(tangles.iter().map(Violation::from))
                .collect(),
        )
    }
}

impl Tangles {
    fn component<'graph>(
        node: &'graph Node<'graph>,
        reached: &BTreeSet<&'graph Node<'graph>>,
        reach: &BTreeMap<&'graph Node<'graph>, BTreeSet<&'graph Node<'graph>>>,
    ) -> BTreeSet<&'graph Node<'graph>> {
        reached
            .iter()
            .copied()
            .filter(|other| reach.get(other).and_then(|back| back.get(node)).is_some())
            .chain([node])
            .collect()
    }
}

impl Lint for Tangles {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Self::report_graph(&Facts::of_files(files), &Transitions::of_files(files))
    }
}
