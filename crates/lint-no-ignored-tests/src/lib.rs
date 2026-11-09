mod skip_marks;
mod test_gates;

use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::{Headline, Lint};
use lint_file::FileLint;
use lint_layout::LibraryTests;
use lint_placement::Role;
use skip_marks::SkipMarks;
use test_gates::TestGates;

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
    const UNRUN: Message = Message::new(
        Problem::new("#[cfg(test)] code in a target that is never built for tests, so it never runs"),
        Fix::new("move the tests into the crate's tests directory and test through the public API"),
    );
}

impl NoIgnoredTests {
    fn never_tested(file: &SourceFile) -> Option<&SourceFile> {
        match file.placement().role() {
            Role::BinaryRoot => file.switched_off().map(|_switched_off| file),
            Role::BuildScript => Some(file),
            Role::LibraryRoot | Role::Other | Role::Target => {
                Some(file).filter(|library| library.placement().package().library_tests() == LibraryTests::SwitchedOff)
            }
        }
    }
}

impl FileLint for NoIgnoredTests {
    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        file.shipped()
            .and_then(Self::never_tested)
            .map(|shipped| TestGates::lines_in(shipped.syntax()))
            .unwrap_or_default()
            .into_iter()
            .map(|line| Violation::new(Site::new(file.path(), line), Self::UNRUN))
            .collect()
    }
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
                .chain(Self::in_shipped(files))
                .collect(),
        )
    }
}
