mod skip_marks;

use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::{Headline, Lint};
use skip_marks::SkipMarks;

#[derive(Debug)]
pub struct NoIgnoredTests;

impl NoIgnoredTests {
    const UNTESTED: Message = Message::new(
        Problem::new("test target switched off in Cargo.toml"),
        Fix::new("remove `test = false` and make the tests pass"),
    );
    const SKIPPED: Message = Message::new(
        Problem::new("test skipped by an attribute"),
        Fix::new("remove the attribute and make the test pass, or delete the test"),
    );
}

impl Lint for NoIgnoredTests {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new("every test runs"),
            files
                .iter()
                .flat_map(|file| {
                    let target = file.switched_off().map(|path| Violation::new(Site::of_file(path), Self::UNTESTED));
                    SkipMarks::lines_in(file.syntax())
                        .into_iter()
                        .map(move |line| Violation::new(Site::new(file.path(), line), Self::SKIPPED))
                        .chain(target)
                })
                .collect(),
        )
    }
}
