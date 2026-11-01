mod finding;
mod shape;
mod signature_index;

use finding::Finding;
use signature_index::SignatureIndex;

use syn::Ident;
use syn::visit::Visit;

use crate::Lint;
use lint_core::{LineNumber, Name, Report, Site, SourceFile, Violation};

#[derive(Debug)]
pub struct DistinctSignatures;

impl Lint for DistinctSignatures {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "two functions on one type with the same signature hide a missing type or type parameter",
            files.iter().filter(|file| file.is_shipped()).flat_map(Self::violations).collect(),
        )
    }
}

impl DistinctSignatures {
    fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut index = SignatureIndex::default();
        index.visit_file(file.syntax());
        index
            .shared()
            .into_iter()
            .map(|(shape, idents)| {
                let line = idents.first().map_or(LineNumber::FIRST, |ident| LineNumber::from(ident.span()));
                Violation::new(Site::new(file.path(), line), Finding::new(Self::named(&idents), shape))
            })
            .collect()
    }

    fn named(idents: &[Ident]) -> Vec<Name> {
        idents.iter().map(Name::from).collect()
    }
}
