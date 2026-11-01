mod skip_marks;

use lint_core::{Lint, Report, Site, SourceFile, Violation};
use skip_marks::SkipMarks;

#[derive(Debug)]
pub struct NoIgnoredTests;

impl NoIgnoredTests {
    const FIX: &str = "test skipped by an attribute; remove the attribute and make the test pass, or delete the test";
}

impl Lint for NoIgnoredTests {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "every test runs",
            files
                .iter()
                .flat_map(|file| {
                    SkipMarks::lines_in(file.syntax())
                        .into_iter()
                        .map(move |line| Violation::new(Site::new(file.path(), line), Self::FIX))
                })
                .collect(),
        )
    }
}
