mod ports;
mod reader;

use graph::{Ground, Model, Reach, Region};
use lint_core::{Headline, Report};
use lint_file::ModelLint;

use ports::Ports;
use reader::{Found, Reader};

#[derive(Debug)]
pub struct PortBatching;

impl ModelLint for PortBatching {
    fn report(model: &Model) -> Report<'_> {
        let ports = Ports::from(model);
        let reader = Reader::new(model, &ports);
        let found: Found<'_> = model
            .functions()
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() != Reach::Test)
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).ground() != Ground::Ports)
            .flat_map(|placed| reader.found(placed))
            .collect();
        Report::new(Headline::new("ports are called once per batch, not once per element"), found.into_iter().collect())
    }
}
