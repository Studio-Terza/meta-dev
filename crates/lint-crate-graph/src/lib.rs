mod links;

use code_model::Model;
use lint_core::{Headline, ModelLint, Report};

use links::Links;

#[derive(Debug)]
pub struct CrateGraph;

impl ModelLint for CrateGraph {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new("a crate whose items form one big cycle is split"),
            model
                .holdings()
                .map(Links::from)
                .chain(model.implementations().map(Links::from))
                .chain(model.functions().map(Links::from))
                .chain(model.uses().map(Links::from))
                .collect::<Links<'_>>()
                .violations(model),
        )
    }
}
