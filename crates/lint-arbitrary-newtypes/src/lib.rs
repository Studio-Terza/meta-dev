mod ungenerated;

use lint_core::{Headline, Lint, Report, SourceFile};
use lint_file::FileLint;
use ungenerated::Ungenerated;

#[derive(Debug)]
pub struct ArbitraryNewtypes;

impl Lint for ArbitraryNewtypes {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(Headline::new("a newtype that owns its value derives Arbitrary"), Ungenerated::in_shipped(files))
    }
}
