mod export;
mod finding;
mod foreign_references;

use export::Export;
use finding::Finding;
use foreign_references::ForeignReferences;
use lint_core::{Headline, Parsed};
use lint_file::{ContextLint, Reads};

use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_placement::CrateName;

#[derive(Debug)]
pub struct PublicSurface;

#[derive(Debug, Default)]
pub struct References(ForeignReferences);

impl FromIterator<References> for References {
    fn from_iter<Parts: IntoIterator<Item = References>>(parts: Parts) -> References {
        References(parts.into_iter().map(|part| part.0).fold(ForeignReferences::default(), ForeignReferences::absorb))
    }
}

impl ContextLint for PublicSurface {
    const HEADLINE: Headline = Headline::new("a crate exports exactly what another crate uses");
    const READS: Reads = Reads::Every;

    type Part<'head> = References;
    type Context<'head> = References;

    fn contributed(parsed: Parsed<'_, '_>) -> References {
        let file = parsed.file();
        let found = file
            .shipped()
            .map_or_else(|| ForeignReferences::in_own_tests(file), |shipped| Some(ForeignReferences::in_file(shipped)));
        References(found.unwrap_or_default())
    }

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, references: &References) -> Vec<Violation<'file>> {
        let crate_name = CrateName::from(file.head().placement().crate_root());
        Export::in_root(file)
            .into_iter()
            .filter(|export| references.0.unnamed(&crate_name, export).is_some())
            .map(|export| {
                Violation::new(
                    Site::new(file.head().path(), export.line()),
                    Finding::UnnamedExport { name: export.name() },
                )
            })
            .collect()
    }
}
