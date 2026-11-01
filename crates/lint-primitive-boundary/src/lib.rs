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
            Headline::new("primitives appear only at a type's boundary"),
            files
                .iter()
                .filter(|file| file.is_shipped() && !support.contains(&file.path().to_owned()))
                .flat_map(|file| {
                    Some(file)
                        .filter(|file| !Gate::BITS.admits(file))
                        .into_iter()
                        .flat_map(NumberSites::violations)
                        .chain(TextSites::violations(file))
                        .chain(
                            Some(file)
                                .filter(|file| !Gate::PORTS.admits(file))
                                .into_iter()
                                .flat_map(StringSites::exposed_in),
                        )
                })
                .collect(),
        )
    }
}
