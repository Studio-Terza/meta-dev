mod ports;
mod reader;

use code_model::Model;
use lint_core::{Headline, ModelLint, Report};

use ports::Ports;
use reader::Reader;

#[derive(Debug)]
pub struct PortBatching;

impl ModelLint for PortBatching {
    fn report(model: &Model) -> Report<'_> {
        let ports = Ports::from(model);
        Report::new(
            Headline::new("ports are called once per batch, not once per element"),
            Reader::new(model, &ports).violations(),
        )
    }
}
