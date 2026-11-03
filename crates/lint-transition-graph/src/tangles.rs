mod tangle;

use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use lint_core::{Headline, Lint, Report, SourceFile, Violation};
use transition_facts::Transitions;
use type_facts::Facts;

use crate::GraphLint;
use crate::end::{Node, Step};
use crate::graph::{Arc, Graph, Label};
use crate::summary::Summary;
use tangle::{Tally, Tangle};

#[derive(Debug)]
pub struct Tangles;

impl GraphLint for Tangles {
    fn report_graph(facts: &Facts, transitions: &Transitions) -> Report {
        let graph = Graph::built(facts, transitions);
        let leaving: BTreeMap<&Node, Vec<&Arc<'_>>> = graph
            .leaving()
            .into_iter()
            .map(|(node, arcs)| {
                (node, arcs.into_iter().filter(|arc| AsRef::<Label>::as_ref(*arc).lifted().is_none()).collect())
            })
            .collect();
        let reach: BTreeMap<&Node, BTreeSet<&Node>> =
            leaving.keys().map(|node| (*node, Self::reached(node, &leaving))).collect();
        let groups: BTreeSet<BTreeSet<&Node>> = reach
            .iter()
            .map(|(node, reached)| Self::component(node, reached, &reach))
            .filter(|group| group.len() > 1)
            .collect();
        let tangles =
            Tangle::ranked(groups.into_iter().map(|members| Tangle::closed(members, &leaving, &graph)).collect());
        Report::new(
            Headline::new(
                "report only, tangles: strongly connected groups of domain types along written edges, trait impls and store operations; the core should be acyclic (D11), with cycles only as lifecycles declared through states! (whose types the macro generates, so they are not nodes here) or as round trips under a roundtrip! law; a group of two joined only by conversions is listed as a round trip",
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
        node: &'graph Node,
        reached: &BTreeSet<&'graph Node>,
        reach: &BTreeMap<&'graph Node, BTreeSet<&'graph Node>>,
    ) -> BTreeSet<&'graph Node> {
        reached
            .iter()
            .copied()
            .filter(|other| reach.get(other).and_then(|back| back.get(node)).is_some())
            .chain([node])
            .collect()
    }

    fn reached<'graph>(
        start: &'graph Node,
        leaving: &BTreeMap<&'graph Node, Vec<&'graph Arc<'_>>>,
    ) -> BTreeSet<&'graph Node> {
        let mut seen: BTreeSet<&'graph Node> = BTreeSet::new();
        iter::successors(Some(BTreeSet::from([start])), |frontier: &BTreeSet<&'graph Node>| {
            let fresh: BTreeSet<&'graph Node> = frontier
                .iter()
                .flat_map(|node| leaving.get(node).into_iter().flatten())
                .map(|arc| AsRef::<Step>::as_ref(*arc).reached())
                .filter(|reached| seen.insert(*reached))
                .collect();
            fresh.first().map(|_| fresh.clone())
        })
        .skip(1)
        .flatten()
        .collect()
    }
}

impl Lint for Tangles {
    fn report(files: &[SourceFile]) -> Report {
        Self::report_graph(&Facts::of_files(files), &Transitions::of_files(files))
    }
}
