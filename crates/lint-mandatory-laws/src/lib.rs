mod coverage;
mod facts;
mod need;

use coverage::Coverage;
use facts::Facts;
use facts::Obligation;
use lint_core::Lint;
use lint_core::Report;
use lint_core::SourceFile;

#[derive(Debug)]
pub struct MandatoryLaws;

impl Lint for MandatoryLaws {
    fn report(files: &[SourceFile]) -> Report {
        let coverage = Coverage::read_all(files);
        Report::new(
            "conversions and orders have laws",
            files
                .iter()
                .filter(|file| file.is_shipped())
                .flat_map(|file| Facts::collect_from(file.syntax()).obligations(file.path()))
                .filter(|obligation| !coverage.covers(obligation.need(), obligation.subject()))
                .map(Obligation::violation)
                .collect(),
        )
    }
}
