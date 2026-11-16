mod repeated;
mod shaped;

use graph::Model;
use lint_core::{Bound, Headline, Report, Violation};
use lint_file::ModelLint;

use repeated::Repeated;
use shaped::{Inners, Shaped};

#[derive(Debug)]
pub struct RepeatedMethods;

impl ModelLint for RepeatedMethods {
    fn report(model: &Model) -> Report<'_> {
        let inners = Inners::read(model);
        let methods = model.functions().filter_map(|placed| Shaped::placed(placed, &inners));
        let found: Vec<Violation<'_>> = Shaped::grouped(methods)
            .into_iter()
            .filter_map(|members| Repeated::flagged(model, members, Self::TYPES_PER_SHAPE))
            .collect();
        Report::new(
            Headline::new(
                "one public method shape on many types is repeated; use AsRef<T> for a stored field or From<&T> for a computed value, with a newtype, or a derive",
            ),
            found,
        )
    }
}

impl RepeatedMethods {
    const TYPES_PER_SHAPE: Bound = Bound::Most(2);
}
