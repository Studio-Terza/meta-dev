mod coverage;
mod facts;
mod subject;

use coverage::Coverage;
use facts::Facts;
use graph::Model;
use lint_core::{Headline, Report};
use lint_file::ModelLint;

#[derive(Debug)]
pub struct MandatoryLaws;

impl ModelLint for MandatoryLaws {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new("conversions and stored records have laws"),
            Facts::from(model).unmet(&Coverage::from(model)),
        )
    }
}
