mod links;

use graph::Model;
use lint_core::{Headline, ModelLint, Report};

use links::Links;

#[derive(Debug)]
pub struct CrateGraph;

impl ModelLint for CrateGraph {
    fn report(model: &Model) -> Report<'_> {
        Report::new(Headline::new("a crate whose items form one big cycle is split"), Links::from(model).violations())
    }
}
