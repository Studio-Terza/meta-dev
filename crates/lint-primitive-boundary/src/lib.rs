mod finding;
mod judged;
mod legacy;
mod lexicon;
mod lookups;
mod signatures;
mod texts;

use code_model::Model;
use lint_core::{Headline, ModelLint, Report, Violation};

use legacy::Listing;
use lexicon::Lexicon;
use lookups::Lookups;
use signatures::Signatures;
use texts::Texts;

#[derive(Debug)]
pub struct PrimitiveBoundary;

#[derive(Debug)]
pub struct WidenedPrimitiveBoundary;

impl PrimitiveBoundary {
    fn listed(model: &Model, listing: Listing) -> Vec<Violation<'_>> {
        let lexicon = Lexicon::new();
        let lookups = Lookups::read(model);
        let signatures = Signatures::new(&lexicon, &lookups).judged(model);
        let texts = Texts::doored(&lexicon, model).judged(model);
        let mut listed: Vec<Violation<'_>> =
            signatures.into_iter().chain(texts).filter_map(|judged| judged.listed(listing)).collect();
        listed.sort_by(|left, right| left.site().cmp(right.site()));
        listed
    }
}

impl ModelLint for PrimitiveBoundary {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new("primitives appear in signatures and fields only inside a boundary"),
            PrimitiveBoundary::listed(model, Listing::Blocking),
        )
    }
}

impl ModelLint for WidenedPrimitiveBoundary {
    fn report(model: &Model) -> Report<'_> {
        Report::new(
            Headline::new(
                "report only, primitive_boundary widened: the same rule over the code the gate does not yet block (tests, examples, benches, support and cfg(test) modules, macro bodies and std macro arguments, enum and union generics, type aliases, consts, statics, associated types, aliases declared in another file, core::primitive and <T as Trait>::X spellings, hand-written new, try_new and into_inner, and one-field structs of a collection or Option); fix passes clear these, then they join primitive_boundary",
            ),
            PrimitiveBoundary::listed(model, Listing::Widened),
        )
    }
}
