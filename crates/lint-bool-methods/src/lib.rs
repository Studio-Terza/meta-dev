mod returns;

use lint_core::{Headline, Lint, Report, SourceFile, Violation};
use lint_file::{FileLint, Reads};
use returns::Returns;

#[derive(Debug)]
pub struct BoolMethods;

impl Lint for BoolMethods {
    fn report(files: &[SourceFile]) -> Report<'_> {
        <Self as FileLint>::reported(files)
    }
}

impl FileLint for BoolMethods {
    const HEADLINE: Headline =
        Headline::new("a function returning bool forgets what it found and makes the caller branch on a flag");
    const READS: Reads = Reads::Shipped;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        Vec::from(Returns::from(file))
    }
}
