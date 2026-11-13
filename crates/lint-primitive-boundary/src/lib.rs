mod finding;
mod lexicon;
mod scope;
mod signatures;
mod texts;

use graph::Model;
use lint_core::{Headline, ModelLint, Report, Violation};

use lexicon::Lexicon;
use scope::Scope;
use signatures::Signatures;
use texts::Texts;

#[derive(Debug)]
pub struct PrimitiveBoundary;

impl ModelLint for PrimitiveBoundary {
    fn report(model: &Model) -> Report<'_> {
        let lexicon = Lexicon::new();
        let scope = Scope::read(model);
        let mut found: Vec<Violation<'_>> =
            Signatures::new(&lexicon, &scope).found(model).into_iter().chain(Texts::new(&scope).found(model)).collect();
        found.sort_by(|left, right| left.site().path().cmp(right.site().path()));
        Report::new(Headline::new("primitives appear in signatures and fields only inside a boundary"), found)
    }
}
