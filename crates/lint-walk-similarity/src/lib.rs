mod candidate;
mod walk;

use lint_core::{Headline, Lint, Report, Site, SourceFile, Violation};
use type_facts::Facts;

use candidate::Candidate;
use walk::Walk;

#[derive(Debug)]
pub struct WalkSimilarity;

impl Lint for WalkSimilarity {
    fn report(files: &[SourceFile]) -> Report {
        let facts = Facts::of_files(files);
        let walks: Vec<Walk<'_>> = facts.types().iter().filter_map(Walk::walked).collect();
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
                .map(|candidate| Violation::new(AsRef::<Site>::as_ref(&candidate).clone(), candidate))
                .collect(),
        )
    }
}
