mod error_types;
mod finding;
mod narrowed;

use graph::{Function, Model, Placed};
use lint_core::{Headline, ModelLint, Report, Violation};

use error_types::ErrorTypes;
use finding::Finding;
use narrowed::Narrowed;

#[derive(Debug)]
pub struct NarrowedReturns;

impl ModelLint for NarrowedReturns {
    fn report(model: &Model) -> Report<'_> {
        let errors = ErrorTypes::from(model);
        Report::new(
            Headline::new(
                "a function returning bool, Result<(), E> or Option<E> checks without keeping what it found; parse instead",
            ),
            model.functions().filter_map(|found| Self::flagged(model, &errors, found)).collect(),
        )
    }
}

impl NarrowedReturns {
    fn flagged<'model>(
        model: &Model,
        errors: &ErrorTypes,
        found: Placed<'model, &'model Function>,
    ) -> Option<Violation<'model>> {
        let function = found.entity().called()?;
        let narrowed = Narrowed::returned(model, errors, function.signed_here()?.output())?;
        Some(Violation::new(found.site(), Finding::new(function.as_ref(), narrowed)))
    }
}
