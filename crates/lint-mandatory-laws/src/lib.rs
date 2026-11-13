mod coverage;
mod facts;
mod subject;

use coverage::Coverage;
use facts::Facts;
use graph::Model;
use lint_core::{Headline, ModelLint, Report};

#[derive(Debug)]
pub struct MandatoryLaws;

impl ModelLint for MandatoryLaws {
    fn report(model: &Model) -> Report<'_> {
        Report::new(Headline::new("conversions have roundtrip laws"), Facts::from(model).unmet(&Coverage::from(model)))
    }
}
