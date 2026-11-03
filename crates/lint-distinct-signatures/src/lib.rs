mod signature_index;

use signature_index::SignatureIndex;

use syn::visit::Visit;

use lint_core::Lint;
use lint_core::{Headline, Report, SourceFile, Violation};
use lint_file::FileLint;

#[derive(Debug)]
pub struct DistinctSignatures;

impl Lint for DistinctSignatures {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Self::shipped(
            files,
            Headline::new("two functions on one type with the same signature hide a missing type or type parameter"),
        )
    }
}

impl FileLint for DistinctSignatures {
    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let mut index = SignatureIndex::default();
        index.visit_file(file.syntax());
        index.violations(file.path()).collect()
    }
}
