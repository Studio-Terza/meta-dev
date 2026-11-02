mod assertions;

use assertions::Assertions;
use lint_core::{Headline, Lint, Report, SourceFile};

#[derive(Debug)]
pub struct LongExactStrings;

impl Lint for LongExactStrings {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            Headline::new("assertions do not pin long text"),
            files.iter().flat_map(Assertions::violations).collect(),
        )
    }
}
