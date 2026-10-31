mod bag_candidates;
mod finding;
mod name_uses;

use crate::Lint;
use bag_candidates::BagCandidates;
use finding::Finding;
use name_uses::NameUses;

use lint_core::Name;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;

#[derive(Debug)]
pub struct NoParameterBags;

impl Lint for NoParameterBags {
    fn report(files: &[SourceFile]) -> Report {
        let shipped: Vec<&SourceFile> = files.iter().filter(|file| file.is_shipped()).collect();
        let uses: Vec<NameUses> = shipped.iter().map(|file| NameUses::in_file(file)).collect();
        Report::new(
            "a struct with no logic of its own that one other file consumes is a parameter bag",
            shipped
                .iter()
                .enumerate()
                .flat_map(|(index, file)| {
                    let candidates = BagCandidates::in_file(file);
                    candidates
                        .candidates()
                        .filter(|candidate| Self::has_one_consumer_besides_index(&uses, index, candidate.name()))
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
    fn has_one_consumer_besides_index(uses: &[NameUses], index: usize, name: &Name) -> bool {
        uses.iter().enumerate().filter(|(other, file_uses)| *other != index && file_uses.mentions(name)).count() == 1
    }
}
