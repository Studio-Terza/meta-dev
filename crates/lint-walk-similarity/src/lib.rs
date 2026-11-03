mod candidate;
mod walk;

use std::collections::BTreeSet;

use lint_core::{Headline, Lint, Report, Site, SourceFile, Violation};
use lint_layout::Name;
use type_facts::{Facts, FactsLint};

use candidate::Candidate;
use walk::Walk;

#[derive(Debug)]
pub struct WalkSimilarity;

impl FactsLint for WalkSimilarity {
    fn report_facts<'file>(facts: &Facts<'file>) -> Report<'file> {
        let local: BTreeSet<Name> = facts.traits().iter().map(|found| AsRef::<Name>::as_ref(found).clone()).collect();
        let walks: Vec<Walk<'_, 'file>> = facts
            .types()
            .iter()
            .filter_map(|found| facts.declared_once(found))
            .filter_map(|found| Walk::walked(found, &local))
            .collect();
        Report::new(
            Headline::new(
                "report only, two types with near-identical outgoing methods and trait impls are candidates to unify; types deriving Newtype are left out, since two newtypes over one inner type are distinct nodes on purpose",
            ),
            walks
                .iter()
                .enumerate()
                .flat_map(|(at, first)| {
                    walks.iter().skip(at + 1).filter_map(|second| Candidate::between(first, second))
                })
                .map(|candidate| Violation::new(*AsRef::<Site<'file>>::as_ref(&candidate), candidate))
                .collect(),
        )
    }
}

impl Lint for WalkSimilarity {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Self::report_facts(&Facts::of_files(files))
    }
}
