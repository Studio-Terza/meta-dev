mod bag_candidates;
mod finding;
mod name_uses;
mod root_exports;

use bag_candidates::BagCandidates;
use finding::Finding;
use lint_core::{Headline, Lint};
use name_uses::NameUses;
use root_exports::RootExports;

use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_layout::Name;

#[derive(Debug)]
pub struct NoParameterBags;

impl Lint for NoParameterBags {
    fn report(files: &[SourceFile]) -> Report {
        let shipped: Vec<&SourceFile> = files.iter().filter(|file| file.is_shipped()).collect();
        let uses: Vec<NameUses> = shipped.iter().map(|file| NameUses::in_file(file)).collect();
        let exports = RootExports::in_files(&shipped);
        Report::new(
            Headline::new("a struct with no logic of its own that one other file consumes is a parameter bag"),
            shipped
                .iter()
                .zip(&uses)
                .flat_map(|(file, own)| {
                    let candidates = BagCandidates::in_file(file);
                    candidates
                        .candidates()
                        .filter(|candidate| !exports.contains(file, candidate.name()))
                        .filter(|candidate| Self::has_one_consumer_besides(&uses, own, candidate.name()))
                        .map(|candidate| {
                            Violation::new(
                                Site::new(file.path(), candidate.line()),
                                Finding::ParameterBag { name: candidate.name().clone() },
                            )
                        })
                        .collect::<Vec<Violation>>()
                })
                .collect(),
        )
    }
}

impl NoParameterBags {
    fn has_one_consumer_besides(uses: &[NameUses], own: &NameUses, name: &Name) -> bool {
        uses.iter()
            .filter(|file_uses| {
                !std::ptr::eq(*file_uses, own) && file_uses.shares_crate(own) && file_uses.mentions(name)
            })
            .count()
            == 1
    }
}
