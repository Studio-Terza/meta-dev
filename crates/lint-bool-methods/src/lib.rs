mod finding;
mod returns;

use finding::Finding;
use lint_core::FileLint;
use lint_core::{Headline, Lint, Report, SourceFile};
use returns::Returns;

#[derive(Debug)]
pub struct BoolMethods;

impl Lint for BoolMethods {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            Headline::new("a function returning bool forgets what it found and makes the caller branch on a flag"),
            files.iter().filter_map(SourceFile::shipped).flat_map(Returns::violations).collect(),
        )
    }
}
