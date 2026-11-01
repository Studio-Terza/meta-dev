use std::fmt;
use std::path::PathBuf;

use lint_core::{Lint, Report, SourceFile};

use crate::case::Case;
use crate::source::Source;

#[derive(Debug)]
pub struct Sources(&'static [Source]);

impl Sources {
    #[must_use]
    pub fn file(path: impl Into<PathBuf>, text: &str) -> SourceFile {
        match SourceFile::parse(path, text.to_string()) {
            Ok(file) => file,
            Err(failure) => unreachable!("fixture source parses: {failure}"),
        }
    }

    #[must_use]
    pub fn lines(report: &Report) -> Vec<String> {
        report.violations().iter().map(|violation| violation.site().to_string()).collect()
    }

    #[must_use]
    pub fn mismatches<Checked: Lint>(cases: &[Case]) -> Vec<String> {
        cases
            .iter()
            .filter_map(|case| {
                let sources = Sources(case.sources());
                let files: Vec<SourceFile> =
                    sources.0.iter().map(|source| Self::file(source.path(), source.text())).collect();
                let found = Self::lines(&Checked::report(&files));
                (found != case.flagged()).then(|| format!("{} ({sources}) flagged {found:?}", case.name()))
            })
            .collect()
    }
}

impl fmt::Display for Sources {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|source| write!(formatter, "{} ", source.path().display()))
    }
}
