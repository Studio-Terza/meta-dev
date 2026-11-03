mod guards;

use guards::Guards;
use lint_core::{Headline, Lint, Report, SourceFile};

#[derive(Debug)]
pub struct GuardMustUse;

impl Lint for GuardMustUse {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let shipped: Vec<&SourceFile> = files.iter().filter_map(SourceFile::shipped).collect();
        Report::new(Headline::new("guard types are #[must_use]"), Guards::scanned(&shipped).violations())
    }
}
