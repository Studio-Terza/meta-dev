mod finding;
mod language;

use std::path::Path;

use finding::Finding;
use language::Language;
use lint_core::{Headline, RepoFile, Report, Site, Violation};

#[derive(Debug)]
pub struct RustOnly;

impl RustOnly {
    const ASSETS: &'static str = "assets";

    #[must_use]
    pub fn report(files: &[RepoFile]) -> Report {
        Report::new(
            Headline::new("logic is Rust; web assets live under a directory named assets"),
            files
                .iter()
                .filter(|file| !file.lies_under(Path::new(Self::ASSETS)))
                .filter_map(|file| {
                    Language::detect(file.shown())
                        .map(|language| Violation::new(Site::of_file(file.shown()), Finding::new(language)))
                })
                .collect(),
        )
    }
}
