mod comparison;
mod family;

use std::collections::BTreeMap;

use lint_core::{Headline, Lint, Report, SourceFile};
use type_facts::{Facts, Transition};

use crate::end::{Ends, Key};
use crate::workspace_types::WorkspaceTypes;
use comparison::Comparison;
use family::Family;

#[derive(Debug)]
pub struct DuplicateTransitions;

impl Lint for DuplicateTransitions {
    fn report(files: &[SourceFile]) -> Report {
        let facts = Facts::of_files(files);
        let types = WorkspaceTypes::declared(facts.types());
        let ends: Vec<Ends> = facts.transitions().iter().map(|transition| types.ends(transition)).collect();
        let keyed = facts.transitions().iter().zip(&ends).fold(
            BTreeMap::new(),
            |mut keys: BTreeMap<Key, Vec<&Transition>>, (transition, found)| {
                found.key().into_iter().for_each(|key| keys.entry(key).or_default().push(transition));
                keys
            },
        );
        let families: Vec<Family<'_>> =
            keyed.into_iter().filter_map(|(key, found)| Family::repeated(key, found)).collect();
        Report::new(
            Headline::new(
                "report only, duplicate transitions: input types joined to one output type by more than one function, method or trait impl in shipped code, compared only where every type is a workspace type; keep one, a standard conversion (From, TryFrom, FromStr) where it fits, and route the others through it",
            ),
            Comparison::of_families(&ends, &families)
                .violation(facts.transitions())
                .into_iter()
                .chain(families.iter().filter_map(Family::violation))
                .collect(),
        )
    }
}
