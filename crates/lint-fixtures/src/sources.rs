mod case;
mod flagged;
mod lines;
mod source;
mod text;

use std::fmt;
use std::path::PathBuf;

use lint_core::{Report, SourceFile, SourceText};
use newtype::Newtype;

use crate::checks::Checks;

pub use case::{Case, Title};
pub use flagged::Flagged;
pub use lines::Lines;
pub use source::{Located, Source};
pub use text::Text;

#[derive(Debug)]
pub struct Sources(&'static [Source]);

impl Sources {
    #[must_use]
    pub fn file(path: impl Into<PathBuf>, text: Text<'_>) -> SourceFile {
        match SourceFile::parse(path, SourceText::new(text.into_inner().to_owned())) {
            Ok(file) => file,
            Err(failure) => unreachable!("fixture source parses: {failure}"),
        }
    }

    #[must_use]
    pub fn lines(report: &Report<'_>) -> Lines {
        Lines::new(report.violations().iter().map(|violation| violation.site().to_string()).collect())
    }

    #[must_use]
    pub fn parsed(sources: &[Source]) -> Vec<SourceFile> {
        sources.iter().map(|source| Self::file(source.path(), source.text())).collect()
    }

    #[must_use]
    pub fn flagged<Checked: Checks>(sources: &[Source]) -> Lines {
        Checked::flagged(&Self::parsed(sources))
    }

    #[must_use]
    pub fn mismatches<Checked: Checks>(cases: &[Case]) -> Lines {
        Lines::new(
            cases
                .iter()
                .filter_map(|case| {
                    let sources = Self(case.sources());
                    let found = Self::flagged::<Checked>(sources.0);
                    (found != Lines::from(case.flagged()))
                        .then(|| format!("{} ({sources}) flagged {found}", case.name()))
                })
                .collect(),
        )
    }
}

impl fmt::Display for Sources {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|source| write!(formatter, "{} ", source.path().display()))
    }
}
