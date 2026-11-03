mod comparison;
mod family;

use std::collections::BTreeMap;

use lint_core::{Headline, Lint, Report, SourceFile};
use transition_facts::{Transition, Transitions};
use type_facts::Facts;

use crate::GraphLint;
use crate::edge::{Edge, Role};
use crate::end::Key;
use crate::workspace_types::WorkspaceTypes;
use comparison::Comparison;
use family::Family;

#[derive(Debug)]
pub struct DuplicateTransitions;

impl GraphLint for DuplicateTransitions {
    fn report_graph<'file>(facts: &Facts<'file>, transitions: &Transitions<'file>) -> Report<'file> {
        let types = WorkspaceTypes::declared(facts, transitions);
        let edges: Vec<Edge<'_, 'file>> = transitions.all().map(|found| types.edge(found)).collect();
        let keyed = edges.iter().filter(|edge| edge.role() == Role::Written).fold(
            BTreeMap::new(),
            |mut keys: BTreeMap<Key<'_>, Vec<&Transition<'_, 'file>>>, edge| {
                edge.ends().key().into_iter().for_each(|key| keys.entry(key).or_default().push(edge.transition()));
                keys
            },
        );
        let families: Vec<Family<'_, 'file>> =
            keyed.into_iter().filter_map(|(key, found)| Family::repeated(key, found)).collect();
        Report::new(
            Headline::new(
                "report only, duplicate transitions: input types joined to one output type by more than one written edge (function, method or trait impl) in shipped code, compared only where every type is a domain type; derived lifts and store operations are left out; keep one, a standard conversion (From, TryFrom, FromStr) where it fits, and route the others through it",
            ),
            Comparison::of_families(&edges, &families)
                .violation(transitions)
                .into_iter()
                .chain(families.iter().filter_map(Family::violation))
                .collect(),
        )
    }
}

impl Lint for DuplicateTransitions {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Self::report_graph(&Facts::of_files(files), &Transitions::of_files(files))
    }
}
