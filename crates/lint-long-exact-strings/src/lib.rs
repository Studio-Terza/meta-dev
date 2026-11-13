mod assertions;

use assertions::Assertions;
use lint_core::{Headline, Lint, Report, SourceFile, Violation};
use lint_file::{FileLint, Reads};

#[derive(Debug)]
pub struct LongExactStrings;

impl Lint for LongExactStrings {
    fn report(files: &[SourceFile]) -> Report<'_> {
        <Self as FileLint>::reported(files)
    }
}

impl FileLint for LongExactStrings {
    const HEADLINE: Headline = Headline::new("assertions do not pin long text");
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        Vec::from(Assertions::from(file))
    }
}
