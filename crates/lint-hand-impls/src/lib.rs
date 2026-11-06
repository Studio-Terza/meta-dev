mod finding;

use code_model::{Implementation, Model, Placed};
use lint_core::{Headline, ModelLint, Report, Violation};
use lint_layout::{Whole, Word};

use finding::Finding;

#[derive(Debug)]
pub struct HandImpls;

impl ModelLint for HandImpls {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new("a hand impl of a derivable std trait is prone to subtle bugs; derive it"),
            model.implementations().filter_map(Self::flagged).collect(),
        )
    }
}

impl HandImpls {
    const DERIVABLE: [Word<'static>; 9] = [
        Word::new("Debug"),
        Word::new("Clone"),
        Word::new("Copy"),
        Word::new("PartialEq"),
        Word::new("Eq"),
        Word::new("Hash"),
        Word::new("Default"),
        Word::new("PartialOrd"),
        Word::new("Ord"),
    ];

    fn flagged<'model>(found: Placed<'model, &'model Implementation>) -> Option<Violation<'model>> {
        let implementation = found.entity().written()?;
        let derivable = implementation.implemented()?.std()?.last()?.matches::<Whole>(&Self::DERIVABLE)?;
        Some(Violation::new(found.site(), Finding::new(derivable, implementation.subject())))
    }
}
