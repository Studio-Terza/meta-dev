mod declared;
mod finding;
mod owner;
mod reach;
mod signature_sites;
mod spelling;
mod text_sites;
mod use_aliases;

use std::path::PathBuf;

use lint_core::{Headline, Lint};

use declared::Declared;
use reach::{Reach, Reached};
use signature_sites::SignatureSites;
use text_sites::TextSites;

use lint_core::Gate;
use lint_core::Report;
use lint_core::SourceFile;
use lint_core::SupportModules;
use lint_core::Violation;

#[derive(Debug)]
pub struct PrimitiveBoundary;

#[derive(Debug)]
pub struct WidenedPrimitiveBoundary;

impl PrimitiveBoundary {
    fn reached(files: &[SourceFile], reach: Reach) -> Vec<Violation<'_>> {
        let support: Vec<PathBuf> = files.iter().flat_map(SupportModules::paths).collect();
        let declared = Declared::in_files(files);
        files
            .iter()
            .filter(|file| Gate::BOUNDARY.admits(file).is_none())
            .flat_map(|file| {
                let start = Reach::of_file(file, &support);
                SignatureSites::reached(file, &declared, start).into_iter().chain(TextSites::reached(file, start))
            })
            .filter(|reached| reached.reach() == reach)
            .map(Reached::into_violation)
            .collect()
    }
}

impl Lint for PrimitiveBoundary {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new("primitives appear in signatures and fields only inside a boundary"),
            Self::reached(files, Reach::Blocking),
        )
    }
}

impl Lint for WidenedPrimitiveBoundary {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "report only, primitive_boundary widened: the same rule over the code the gate does not yet block (tests, examples, benches, support and cfg(test) modules, macro bodies, type aliases, consts, statics, associated types, core::primitive and <T as Trait>::X spellings, and one-field structs of a collection or Option); fix passes clear these, then they join primitive_boundary",
            ),
            PrimitiveBoundary::reached(files, Reach::Widened),
        )
    }
}
