mod hidden;

use lint_core::{Headline, Least, Lint, Report, Site, SourceFile, Violation};
use transition_facts::{Transition, Transitions};
use type_facts::Facts;

use crate::GraphLint;
use crate::edge::{Edge, Role};
use crate::graph::Graph;
use crate::summary::Summary;
use hidden::{Hidden, Tally};

#[derive(Debug)]
pub struct HiddenStates;

impl HiddenStates {
    const FLOOR: Least = Least::new(6);
}

impl GraphLint for HiddenStates {
    fn report_graph(facts: &Facts, transitions: &Transitions) -> Report {
        let graph = Graph::built(facts, transitions);
        let written: Vec<&Transition> =
            graph.edges().iter().filter(|edge| edge.role() == Role::Written).map(Edge::transition).collect();
        let mut above: Vec<&Transition> =
            written.iter().copied().filter(|found| Self::FLOOR.allows(found.branches()).is_some()).collect();
        above.sort_by(|left, right| {
            right.branches().cmp(&left.branches()).then_with(|| AsRef::<Site>::as_ref(*left).cmp(right.as_ref()))
        });
        Report::new(
            Headline::new(
                "report only, hidden states: a written edge whose body branches six ways or more (if, loops, match arms beyond the first, && and ||; the ? operator is error propagation, allowed everywhere, and is not a branch) holds unnamed intermediate states between its input and its output; each branch is a state the types do not name; listed by branching, highest first",
            ),
            Summary::placed(
                written.iter().map(|found| AsRef::<Site>::as_ref(*found)).min(),
                Tally::counted(&written, &above),
            )
            .into_iter()
            .chain(above.iter().map(|found| Violation::from(Hidden::new(found))))
            .collect(),
        )
    }
}

impl Lint for HiddenStates {
    fn report(files: &[SourceFile]) -> Report {
        Self::report_graph(&Facts::of_files(files), &Transitions::of_files(files))
    }
}
