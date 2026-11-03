mod export;
mod finding;
mod foreign_references;

use export::Export;
use finding::Finding;
use foreign_references::ForeignReferences;
use lint_core::{Headline, Lint};

use lint_core::BinaryFiles;

use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_placement::CrateName;

#[derive(Debug)]
pub struct PublicSurface;

impl Lint for PublicSurface {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let binaries = BinaryFiles::collect(files);
        let references = files
            .iter()
            .filter_map(|file| {
                file.shipped().map_or_else(
                    || ForeignReferences::in_own_tests(file),
                    |shipped| Some(ForeignReferences::in_file(shipped, &binaries)),
                )
            })
            .fold(ForeignReferences::default(), ForeignReferences::absorb);
        let references = &references;
        Report::new(
            Headline::new("a crate exports exactly what another crate uses"),
            files
                .iter()
                .flat_map(|file| {
                    let crate_name = CrateName::from(file.placement().crate_root());
                    Export::in_root(file)
                        .into_iter()
                        .filter(move |export| references.unnamed(&crate_name, export).is_some())
                        .map(move |export| {
                            Violation::new(
                                Site::new(file.path(), export.line()),
                                Finding::UnnamedExport { name: export.name() },
                            )
                        })
                })
                .collect(),
        )
    }
}
