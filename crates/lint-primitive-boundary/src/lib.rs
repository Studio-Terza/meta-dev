mod finding;
mod newtypes;
mod signature_sites;
mod text_sites;

use lint_core::FileLint;
use lint_core::{Headline, Lint};
use std::path::PathBuf;

use newtypes::Newtypes;
use signature_sites::SignatureSites;
use text_sites::TextSites;

use lint_core::Gate;
use lint_core::Report;
use lint_core::SourceFile;
use lint_core::SupportModules;

#[derive(Debug)]
pub struct PrimitiveBoundary;

impl Lint for PrimitiveBoundary {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let support: Vec<PathBuf> = files.iter().flat_map(SupportModules::paths).collect();
        let newtypes = Newtypes::in_files(files);
        Report::new(
            Headline::new("primitives appear in signatures and fields only inside a boundary"),
            files
                .iter()
                .filter_map(SourceFile::shipped)
                .filter(|file| !support.contains(&file.path().to_owned()) && Gate::BOUNDARY.admits(file).is_none())
                .flat_map(|file| {
                    SignatureSites::violations(file, &newtypes).into_iter().chain(TextSites::violations(file))
                })
                .collect(),
        )
    }
}
