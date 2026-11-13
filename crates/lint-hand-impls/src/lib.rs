mod finding;

use graph::{Domain, External, Implementation, ItemPath, Model, Placed, Resolved};
use lint_core::{Headline, ModelLint, Report, Violation};
use lint_layout::{Whole, Word, words};

use finding::Finding;

#[derive(Debug)]
pub struct HandImpls;

impl ModelLint for HandImpls {
    fn report(model: &Model) -> Report<'_> {
        let guarded: ItemPath = Self::GUARDED.into_iter().collect();
        let declared = model.exported(Domain::Types, &guarded);
        Report::new(
            Headline::new("a hand impl of a derivable trait is prone to subtle bugs; derive it"),
            model.implementations().filter_map(|found| Self::flagged(found, &guarded, declared)).collect(),
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

    fn flagged<'model>(
        found: Placed<'model, &'model Implementation>,
        guarded: &ItemPath,
        declared: Option<&Resolved>,
    ) -> Option<Violation<'model>> {
        let implementation = found.entity().written()?;
        let implemented = implementation.implemented()?;
        let derivable = implemented
            .std()
            .and_then(|path| path.last()?.matches::<Whole>(&Self::DERIVABLE))
            .or_else(|| Self::guarded(implemented, guarded, declared)?.last()?.matches::<Whole>(&Self::GUARDED))?;
        Some(Violation::new(found.site(), Finding::new(derivable, implementation.subject())))
    }

    fn guarded<'path>(
        implemented: &'path Resolved,
        guarded: &'path ItemPath,
        declared: Option<&Resolved>,
    ) -> Option<&'path ItemPath> {
        declared
            .filter(|identity| *identity == implemented)
            .map(|_| guarded)
            .or_else(|| implemented.external().map(External::path).filter(|path| *path == guarded))
    }
}
