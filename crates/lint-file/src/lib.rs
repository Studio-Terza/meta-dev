use lint_core::{SourceFile, Violation};

pub trait FileLint {
    fn violations(file: &SourceFile) -> Vec<Violation<'_>>;

    fn in_shipped(files: &[SourceFile]) -> Vec<Violation<'_>> {
        files.iter().filter_map(SourceFile::shipped).flat_map(Self::violations).collect()
    }
}
