mod finding;
mod raw_paths;

use finding::Finding;
use lint_core::{Headline, Lint, Report, SourceFile};
use lint_paths::Paths;
use raw_paths::RawPaths;

#[derive(Debug)]
pub struct RawResources;

impl Lint for RawResources {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            Headline::new("only ports touch raw resources"),
            RawPaths::found(&Paths::read(files, &RawPaths::FINALS)),
        )
    }
}
