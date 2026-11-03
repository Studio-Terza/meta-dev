mod coverage;
mod facts;
mod package;
mod subject;

use coverage::Coverage;
use facts::Facts;
use lint_core::Report;
use lint_core::SourceFile;
use lint_core::{Headline, Lint, Violation};

#[derive(Debug)]
pub struct MandatoryLaws;

impl Lint for MandatoryLaws {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let coverage = Coverage::read_all(files);
        Report::new(
            Headline::new("conversions and orders have laws"),
            Facts::read_all(files).unmet(&coverage).into_iter().map(Violation::from).collect(),
        )
    }
}
