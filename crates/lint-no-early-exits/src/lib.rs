mod exits;

use exits::Exits;
use lint_core::{Headline, Lint, Report, SourceFile};
use lint_file::FileLint;

#[derive(Debug)]
pub struct NoEarlyExits;

impl Lint for NoEarlyExits {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new("a body leaves through its tail value or ?, never through return, break or continue"),
            files.iter().flat_map(Exits::violations).collect(),
        )
    }
}
