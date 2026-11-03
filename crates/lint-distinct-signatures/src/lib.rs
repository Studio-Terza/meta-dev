mod signature_index;

use signature_index::SignatureIndex;

use syn::visit::Visit;

use lint_core::Lint;
use lint_core::{Headline, Report, SourceFile, Violation};

#[derive(Debug)]
pub struct DistinctSignatures;

impl Lint for DistinctSignatures {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            Headline::new("two functions on one type with the same signature hide a missing type or type parameter"),
            files.iter().filter_map(SourceFile::shipped).flat_map(Self::violations).collect(),
        )
    }
}

impl DistinctSignatures {
    fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut index = SignatureIndex::default();
        index.visit_file(file.syntax());
        index.violations(file.path()).collect()
    }
}
