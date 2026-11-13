mod doubles;

use code_model::Model;
use doubles::Doubles;
use lint_core::{Headline, ModelLint, Report};

#[derive(Debug)]
pub struct FakesAtPorts;

impl ModelLint for FakesAtPorts {
    fn report(model: &Model) -> Report<'_> {
        Report::new(Headline::new("test doubles stand in for ports"), Doubles::from(model).violations(model))
    }
}
