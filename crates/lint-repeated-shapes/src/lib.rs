mod extent;
mod finding;
mod kind;
mod occurrence;
mod scan;

use equivalence::Groups;
use lint_core::{Headline, Lint, Report, SourceFile};

use finding::Finding;
use scan::Scan;

#[derive(Debug)]
pub struct RepeatedShapes;

impl Lint for RepeatedShapes {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let occurrences = files.iter().flat_map(Scan::of_file);
        Report::new(
            Headline::new(
                "a body of 40 tokens or more repeats across the workspace with its names and literals erased; each group says whether its copies are tests, shipped code or both",
            ),
            Groups::grouped(occurrences)
                .ranked(|shape| shape.size())
                .maximal()
                .listed()
                .iter()
                .filter_map(Finding::violation)
                .collect(),
        )
    }
}
