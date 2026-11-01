mod finding;
mod guards;

use finding::Finding;
use guards::Guards;
use lint_core::{Lint, Report, SourceFile};

#[derive(Debug)]
pub struct GuardMustUse;

impl Lint for GuardMustUse {
    fn report(files: &[SourceFile]) -> Report {
        let guards = files.iter().filter(|file| file.is_shipped()).fold(Guards::default(), |mut guards, file| {
            guards.scan(file);
            guards
        });
        Report::new("guard types are #[must_use]", guards.violations())
    }
}
