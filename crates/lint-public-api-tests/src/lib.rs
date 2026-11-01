mod test_code;

use lint_core::{Lint, Report, Site, SourceFile, Violation};
use test_code::TestCode;

#[derive(Debug)]
pub struct PublicApiTests;

impl PublicApiTests {
    const FIX: &str = "test code in src; move it to the package's tests directory and call the public API from there";
}

impl Lint for PublicApiTests {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "tests go through public APIs",
            files
                .iter()
                .filter(|file| file.is_shipped())
                .flat_map(|file| {
                    TestCode::lines_in(file.syntax())
                        .into_iter()
                        .map(move |line| Violation::new(Site::new(file.path(), line), Self::FIX))
                })
                .collect(),
        )
    }
}
