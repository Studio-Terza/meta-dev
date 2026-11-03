use lint_core::{SourceFile, Violation};

pub trait FileLint {
    fn violations(file: &SourceFile) -> Vec<Violation<'_>>;
}
