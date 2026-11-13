mod exits;

use exits::Exits;
use lint_core::{Headline, Lint, Report, SourceFile, Violation};
use lint_file::{FileLint, Reads};

#[derive(Debug)]
pub struct NoEarlyExits;

impl Lint for NoEarlyExits {
    fn report(files: &[SourceFile]) -> Report<'_> {
        <Self as FileLint>::reported(files)
    }
}

impl FileLint for NoEarlyExits {
    const HEADLINE: Headline =
        Headline::new("a body leaves through its tail value or ?, never through return, break or continue");
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        Vec::from(Exits::from(file))
    }
}
