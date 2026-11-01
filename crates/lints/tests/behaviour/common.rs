use lint_core::{Report, SourceFile};
use lints::Lint;

pub struct Source {
    path: &'static str,
    text: &'static str,
}

impl Source {
    pub const fn new(path: &'static str, text: &'static str) -> Source {
        Source { path, text }
    }
}

pub struct Case {
    pub name: &'static str,
    pub sources: &'static [Source],
    pub flagged: &'static [&'static str],
}

pub struct Sources;

impl Sources {
    pub fn file(path: &str, text: &str) -> SourceFile {
        match SourceFile::parse(path, text.to_string()) {
            Ok(file) => file,
            Err(failure) => unreachable!("fixture source parses: {failure}"),
        }
    }

    pub fn lines(report: &Report) -> Vec<String> {
        report.violations().iter().map(|violation| violation.site().to_string()).collect()
    }

    pub fn mismatches<Checked: Lint>(cases: &[Case]) -> Vec<String> {
        cases
            .iter()
            .filter_map(|case| {
                let files: Vec<SourceFile> =
                    case.sources.iter().map(|source| Self::file(source.path, source.text)).collect();
                let found = Self::lines(&Checked::report(&files));
                (found != case.flagged).then(|| format!("{} flagged {found:?}", case.name))
            })
            .collect()
    }
}
