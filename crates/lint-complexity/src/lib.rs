mod crates;

use lint_core::{Headline, Lint, Report, SourceFile};

use crates::Crates;

#[derive(Debug)]
pub struct Complexity;

impl Lint for Complexity {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            Headline::new(
                "report only, cyclomatic complexity per function, as a distribution per crate; a branchy function is a candidate for a standard trait, a generic over an existing trait, or a derive",
            ),
            Crates::grouped(files).violations(),
        )
    }
}
