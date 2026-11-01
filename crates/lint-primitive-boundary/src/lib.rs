mod finding;
mod number_sites;
mod text_sites;

use lint_core::Lint;
use std::path::PathBuf;

use number_sites::NumberSites;
use text_sites::TextSites;

use lint_core::Report;
use lint_core::SourceFile;
use lint_core::SupportModules;

#[derive(Debug)]
pub struct PrimitiveBoundary;

impl Lint for PrimitiveBoundary {
    fn report(files: &[SourceFile]) -> Report {
        let support: Vec<PathBuf> = files.iter().flat_map(SupportModules::paths).collect();
        Report::new(
            "primitives appear only at a type's boundary",
            files
                .iter()
                .filter(|file| file.is_shipped() && !support.contains(&file.path().to_owned()))
                .flat_map(|file| NumberSites::violations(file).into_iter().chain(TextSites::violations(file)))
                .collect(),
        )
    }
}
