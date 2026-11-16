use lint_layout::Name;
use std::path::Path;
mod export;
mod finding;
mod foreign_references;

use export::Export;
use finding::Finding;
use foreign_references::{ForeignReferences, Reading};
use lint_core::{Headline, Parsed};
use lint_file::{ContextLint, Reads, RuledLint};
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
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
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
        ReferenceGathering(Reading::from(<&SourceFile>::from(parsed)))
    }
}

impl RuledLint for PublicSurface {
    fn ruled<'file, 'head: 'file>(file: &'file SourceFile, references: &References) -> Vec<Violation<'file>> {
        Self::unused_exports(file, references)
    }
}

impl PublicSurface {
    fn unused_exports<'file>(file: &'file SourceFile, references: &References) -> Vec<Violation<'file>> {
        let crate_name = CrateName::from(file.head().placement().crate_root());
        Export::in_root(file)
            .into_iter()
            .filter(|export| references.0.unnamed(&crate_name, export).is_some())
            .map(|export| {
                Violation::new(
                    Site::new(AsRef::<Path>::as_ref(file.head()), export.line()),
                    Finding::UnnamedExport { name: AsRef::<Name>::as_ref(&export) },
                )
            })
            .collect()
    }
}
