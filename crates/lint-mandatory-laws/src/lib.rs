mod coverage;
mod facts;
mod need;
mod obligation;
mod package;
mod pair;
mod subject;

use coverage::Coverage;
use facts::Facts;
use lint_core::Lint;
use lint_core::Report;
use lint_core::SourceFile;
use obligation::Obligation;

#[derive(Debug)]
pub struct MandatoryLaws;

impl Lint for MandatoryLaws {
    fn report(files: &[SourceFile]) -> Report {
        let coverage = Coverage::read_all(files);
        Report::new(
            "conversions and orders have laws",
            Facts::read_all(files).unmet(&coverage).into_iter().map(Obligation::violation).collect(),
        )
    }
}
