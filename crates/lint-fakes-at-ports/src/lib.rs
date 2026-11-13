mod doubles;
mod kind;
mod port_traits;

use code_model::Model;
use doubles::Doubles;
pub use kind::Kind;
use lint_core::{Headline, ModelLint, Report};
pub use port_traits::PortTraits;

#[derive(Debug)]
pub struct FakesAtPorts;

impl ModelLint for FakesAtPorts {
    fn report(model: &Model) -> Report<'_> {
        Report::new(Headline::new("test doubles stand in for ports"), Doubles::from(model).violations(model))
    }
}
