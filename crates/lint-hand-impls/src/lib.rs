mod finding;

use code_model::{External, Implementation, ItemPath, Model, Placed};
use lint_core::{Headline, ModelLint, Report, Violation};
use lint_layout::{Whole, Word, words};

use finding::Finding;

#[derive(Debug)]
pub struct HandImpls;

impl ModelLint for HandImpls {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new("a hand impl of a derivable trait is prone to subtle bugs; derive it"),
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
    const GUARDED: [Word<'static>; 2] = words!["newtype", "Guarded"];

    fn flagged<'model>(found: Placed<'model, &'model Implementation>) -> Option<Violation<'model>> {
        let implementation = found.entity().written()?;
        let implemented = implementation.implemented()?;
        let guarded: ItemPath = Self::GUARDED.into_iter().collect();
        let derivable =
            implemented.std().and_then(|path| path.last()?.matches::<Whole>(&Self::DERIVABLE)).or_else(|| {
                implemented
                    .external()
                    .map(External::path)
                    .filter(|path| **path == guarded)?
                    .last()?
                    .matches::<Whole>(&Self::GUARDED)
            })?;
        Some(Violation::new(found.site(), Finding::new(derivable, implementation.subject())))
    }
}
