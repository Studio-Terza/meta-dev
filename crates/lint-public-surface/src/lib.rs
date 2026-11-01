mod export;
mod finding;
mod foreign_references;

use export::Export;
use finding::Finding;
use foreign_references::ForeignReferences;
use lint_core::Lint;

use lint_core::BinaryFiles;

use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;

#[derive(Debug)]
pub struct PublicSurface;

impl Lint for PublicSurface {
    fn report(files: &[SourceFile]) -> Report {
        let binaries = BinaryFiles::collect(files);
        let references = files
            .iter()
            .map(|file| ForeignReferences::in_file(file, &binaries))
            .fold(ForeignReferences::default(), ForeignReferences::absorb);
        Report::new(
            "a crate exports exactly what another crate uses",
            files
                .iter()
                .flat_map(|file| {
                    Export::in_root(file)
                        .into_iter()
                        .filter(|export| references.depends_on(export.crate_name()) && !references.names(export))
                        .map(move |export| {
                            Violation::new(
                                Site::new(file.path(), export.line()),
                                Finding::UnnamedExport { name: export.name().clone() },
                            )
                        })
                })
                .collect(),
        )
    }
}
