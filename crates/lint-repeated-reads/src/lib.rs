mod groups;

use graph::Model;
use lint_core::{Headline, ModelLint, Report};

use groups::Groups;

#[derive(Debug)]
pub struct RepeatedReads;

impl ModelLint for RepeatedReads {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new(
                "report only, a fact is read once: when the implementations of a trait method are run together (the trait is used as dyn, or its methods are listed as values in a const or static table), they do not each compute the same function of the same parameter; compute it once where the method is called and pass it in. Not caught: facts computed by method calls on a parameter, and a fact computed twice inside one type",
            ),
            Groups::found(model).violations(),
        )
    }
}
