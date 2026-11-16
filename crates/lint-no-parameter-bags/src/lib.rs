mod candidate;

use candidate::{Bag, Carriers};
use graph::Model;
use lint_core::{Headline, Report};
use lint_file::ModelLint;

#[derive(Debug)]
pub struct NoParameterBags;

impl NoParameterBags {
    const HEADLINE: Headline =
        Headline::new("a struct with no logic of its own that one other file consumes is a parameter bag");
}

impl ModelLint for NoParameterBags {
    fn report(model: &Model) -> Report<'_> {
        let carriers = Carriers::from(model);
        let consumers = model.consumers();
        let bags = model.items().filter_map(|placed| Bag::found(model, placed));
        Report::new(Self::HEADLINE, bags.filter_map(|bag| bag.bagged(&carriers, &consumers)).collect())
    }
}
