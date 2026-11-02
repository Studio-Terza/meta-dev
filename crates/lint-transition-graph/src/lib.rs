mod count;
mod crate_transitions;
mod duplicates;
mod end;
mod workspace_types;

use std::collections::BTreeMap;

use lint_core::{Headline, Lint, Report, SourceFile};
use lint_placement::CrateName;
use type_facts::{Facts, Transition};

use crate_transitions::CrateTransitions;
use workspace_types::WorkspaceTypes;

pub use duplicates::DuplicateTransitions;

#[derive(Debug)]
pub struct TransitionGraph;

impl Lint for TransitionGraph {
    fn report(files: &[SourceFile]) -> Report {
        let facts = Facts::of_files(files);
        let types = WorkspaceTypes::declared(facts.types());
        let grouped = facts.transitions().iter().fold(
            BTreeMap::new(),
            |mut crates: BTreeMap<&CrateName, Vec<&Transition>>, found| {
                crates.entry(found.as_ref()).or_default().push(found);
                crates
            },
        );
        Report::new(
            Headline::new(
                "report only, the transition graph per crate, types as nodes, functions, methods and trait impls as edges from their input types to their output type; coverage is the share of transitions whose inputs and output are all workspace types, since a bare primitive or unit hides what a transition does; raise it with a newtype, an enum or a standard trait",
            ),
            grouped
                .iter()
                .filter_map(|(name, found)| CrateTransitions::measured(&types, name, found).violation())
                .collect(),
        )
    }
}
