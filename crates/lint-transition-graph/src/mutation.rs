mod crate_mutation;
mod tally;

use lint_core::{Headline, Report};
use transition_facts::Transitions;
use type_facts::Facts;

use crate::GraphLint;
use crate::edge::Edge;
use crate::workspace_types::WorkspaceTypes;
use crate_mutation::CrateMutation;
use tally::Tally;

#[derive(Debug)]
pub struct CoreMutation;

impl GraphLint for CoreMutation {
    fn report_graph<'file>(facts: &Facts<'file>, transitions: &Transitions<'file>) -> Report<'file> {
        let types = WorkspaceTypes::declared(facts, transitions);
        let edges: Vec<Edge<'_, 'file>> = transitions.all().map(|found| types.edge(found)).collect();
        let counted: Vec<&Edge<'_, 'file>> = edges.iter().filter_map(Tally::counts).collect();
        let grouped = Edge::by_crate(counted.iter().copied());
        Report::new(
            Headline::new(
                "report only, mutation in the core: edges outside ports, boundary and store modules that take &mut, through &mut self or a &mut parameter, size the store migration; a signature a foreign trait fixes (fmt, next, hash) is counted apart; store operations, &mut self methods in a store module, are listed with the element transition they apply",
            ),
            grouped
                .iter()
                .filter_map(|(name, found)| CrateMutation::measured(name, found).reported())
                .chain(Tally::counted(counted.iter().copied()).violation(counted.iter().map(|edge| edge.site()).min()))
                .collect(),
        )
    }
}
