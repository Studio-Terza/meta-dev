mod skip_marks;

use lint_core::Fix;
use lint_core::Headline;
use lint_core::Message;
use lint_core::Problem;
use lint_core::SourceFile;
use lint_file::{Reads, SweptLint};
use lint_layout::LibraryTests;
use lint_placement::Role;
use lint_sweep::Finds;
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
    const UNRUN: Message = Message::new(
        Problem::new("#[cfg(test)] code in a target that is never built for tests, so it never runs"),
        Fix::new("move the tests into the crate's tests directory and test through the public API"),
    );
}

impl NoIgnoredTests {
    fn never_tested(file: &SourceFile) -> Option<&SourceFile> {
        match file.head().placement().role() {
            Role::BinaryRoot => file.head().switched_off().map(|_switched_off| file),
            Role::BuildScript => Some(file),
            Role::LibraryRoot | Role::Other | Role::Target => Some(file)
                .filter(|library| library.head().placement().package().library_tests() == LibraryTests::SwitchedOff),
        }
    }
}

impl SweptLint for NoIgnoredTests {
    const HEADLINE: Headline = Headline::new("every test runs");
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        let unrun = file.shipped().and_then(Self::never_tested).map(|_| Vec::new());
        Box::new(SkipMarks::new(file, unrun))
    }
}
