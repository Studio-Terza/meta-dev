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
        let shipped: Vec<&SourceFile> = files.iter().filter_map(SourceFile::shipped).collect();
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
                        .filter(|candidate| exports.exported(file, candidate.name()).is_none())
                        .filter(|candidate| Self::one_consumer_besides(&uses, own, candidate.name()).is_some())
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
    fn one_consumer_besides<'uses>(uses: &'uses [NameUses], own: &NameUses, name: &Name) -> Option<&'uses NameUses> {
        let consumers: Vec<&NameUses> = uses
            .iter()
            .filter(|file_uses| !std::ptr::eq(*file_uses, own))
            .filter_map(|file_uses| file_uses.sharing_crate(own))
            .filter_map(|file_uses| file_uses.mentions(name).map(|_| file_uses))
            .collect();
        match consumers.as_slice() {
            [only] => Some(only),
            _ => None,
        }
    }
}
