mod finding;
mod guards;

use finding::Finding;
use guards::Guards;
use lint_core::{Headline, Lint, Report, SourceFile};

#[derive(Debug)]
pub struct GuardMustUse;

impl Lint for GuardMustUse {
    fn report(files: &[SourceFile]) -> Report {
        let shipped: Vec<&SourceFile> = files.iter().filter(|file| file.is_shipped()).collect();
        let guards = shipped.iter().fold(Guards::default(), |mut guards, file| {
            guards.scan(file);
            guards
        });
        Report::new(Headline::new("guard types are #[must_use]"), guards.violations())
    }
}
