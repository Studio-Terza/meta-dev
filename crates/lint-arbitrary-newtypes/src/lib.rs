mod ungenerated;

use lint_core::FileLint;
use lint_core::{Headline, Lint, Report, SourceFile};
use ungenerated::Ungenerated;

#[derive(Debug)]
pub struct ArbitraryNewtypes;

impl Lint for ArbitraryNewtypes {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new("a newtype that owns its value derives Arbitrary"),
            files.iter().filter_map(SourceFile::shipped).flat_map(Ungenerated::violations).collect(),
        )
    }
}
