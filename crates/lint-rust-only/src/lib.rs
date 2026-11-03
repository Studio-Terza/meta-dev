mod finding;
mod language;
mod place;

use finding::Finding;
use language::Language;
use lint_core::{Headline, Lint, RepoFile, Report, Site, Violation};
use place::Place;

#[derive(Debug)]
pub struct RustOnly;

impl Lint<RepoFile> for RustOnly {
    fn report(files: &[RepoFile]) -> Report<'_> {
        Report::new(
            Headline::new("logic is Rust; web assets live under a directory named assets"),
            files
                .iter()
                .filter_map(|file| {
                    let place = Place::locating(file);
                    Language::detect(file)
                        .and_then(|language| place.refusing(language))
                        .map(|language| Violation::new(Site::of_file(file.shown()), Finding::new(language, place)))
                })
                .collect(),
        )
    }
}
