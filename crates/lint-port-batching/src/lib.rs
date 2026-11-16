use lint_core::Violation;
mod ports;
mod reader;

use graph::Model;
use lint_core::{Headline, Report};
use lint_file::ModelLint;

use ports::Ports;
use reader::Reader;

#[derive(Debug)]
pub struct PortBatching;

impl ModelLint for PortBatching {
    fn report(model: &Model) -> Report<'_> {
        let ports = Ports::from(model);
        Report::new(
            Headline::new("ports are called once per batch, not once per element"),
            Vec::<Violation<'_>>::from(&Reader::new(model, &ports)),
        )
    }
}
