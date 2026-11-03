mod finding;
mod number_sites;
mod string_sites;
mod text_sites;

use lint_core::{Headline, Lint};
use std::path::PathBuf;

use number_sites::NumberSites;
use string_sites::StringSites;
use text_sites::TextSites;

use lint_core::Gate;
use lint_core::Report;
use lint_core::SourceFile;
use lint_core::SupportModules;

#[derive(Debug)]
pub struct PrimitiveBoundary;

impl Lint for PrimitiveBoundary {
    fn report(files: &[SourceFile]) -> Report {
        let support: Vec<PathBuf> = files.iter().flat_map(SupportModules::paths).collect();
        Report::new(
            Headline::new("primitives appear only at a type's boundary or in a boundary module"),
            files
                .iter()
                .filter_map(SourceFile::shipped)
                .filter(|file| !support.contains(&file.path().to_owned()) && Gate::BOUNDARY.admits(file).is_none())
                .flat_map(|file| {
                    NumberSites::violations(file).into_iter().chain(TextSites::violations(file)).chain(
                        Some(file)
                            .filter(|file| Gate::PORTS.admits(file).is_none())
                            .into_iter()
                            .flat_map(StringSites::exposed_in),
                    )
                })
                .collect(),
        )
    }
}
