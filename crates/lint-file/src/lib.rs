use lint_core::{Headline, Report, SourceFile, Violation};

pub trait FileLint {
    fn violations(file: &SourceFile) -> Vec<Violation<'_>>;

    fn shipped(files: &[SourceFile], headline: Headline) -> Report<'_> {
        Report::new(headline, files.iter().filter_map(SourceFile::shipped).flat_map(Self::violations).collect())
    }
}
