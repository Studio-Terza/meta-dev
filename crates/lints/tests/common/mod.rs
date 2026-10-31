use lint_core::{Report, SourceFile};

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
}
