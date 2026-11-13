use lint_core::{Headline, Report, SourceFile, Violation};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reads {
    Shipped,
    Every,
}

impl Reads {
    #[must_use]
    pub fn admits(self, file: &SourceFile) -> Option<&SourceFile> {
        match self {
            Reads::Shipped => file.shipped(),
            Reads::Every => Some(file),
        }
    }
}

pub trait FileLint {
    const HEADLINE: Headline;
    const READS: Reads;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>>;

    fn reported(files: &[SourceFile]) -> Report<'_> {
        let read = files.iter().filter_map(|file| Self::READS.admits(file));
        Report::new(Self::HEADLINE, read.flat_map(Self::violations).collect())
    }
}
