mod export;
mod finding;
mod foreign_references;

use export::Export;
use finding::Finding;
use foreign_references::{ForeignReferences, Reading};
use lint_core::{Headline, Parsed};
use lint_file::{ContextLint, Reads};
use lint_sweep::Judgement;
use lint_sweep::{Gathers, Watching};

use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_placement::CrateName;

#[derive(Debug)]
pub struct PublicSurface;

#[derive(Debug, Default)]
pub struct References(ForeignReferences);

#[derive(Debug)]
pub struct ReferenceGathering(Reading);

impl Gathers<References> for ReferenceGathering {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        self.0.watching()
    }

    fn gathered(self) -> References {
        References(self.0.gathered())
    }
}

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

    type Gathering<'head> = ReferenceGathering;

    fn gathering(parsed: Parsed<'_, '_>) -> ReferenceGathering {
        ReferenceGathering(Reading::from(parsed.file()))
    }

    fn judgement<'run, 'file: 'run, 'head: 'file>(
        file: &'file SourceFile,
        references: &'run References,
    ) -> Judgement<'file, 'run> {
        Judgement::found(Self::judged(file, references))
    }
}

impl PublicSurface {
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
