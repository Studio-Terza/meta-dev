mod crates;

use graph::Model;
use lint_core::{Headline, Report};
use lint_file::ModelLint;

use crates::Crates;

#[derive(Debug)]
pub struct Complexity;

impl ModelLint for Complexity {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new(
                "report only, cyclomatic complexity per function, as a distribution per crate; a branchy function is a candidate for a standard trait, a generic over an existing trait, or a derive",
            ),
            Crates::grouped(model).listed().map(|found| found.violation()).collect(),
        )
    }
}
