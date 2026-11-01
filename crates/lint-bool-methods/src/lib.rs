mod finding;
mod predicates;

use finding::Finding;
use lint_core::{Lint, Report, SourceFile};
use predicates::Predicates;

#[derive(Debug)]
pub struct BoolMethods;

impl Lint for BoolMethods {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "report only, a public is_ or has_ method forgets what it found; return an Option, an enum, or a proof type such as NonEmpty",
            files.iter().filter(|file| file.is_shipped()).flat_map(Predicates::of_file).collect(),
        )
    }
}
