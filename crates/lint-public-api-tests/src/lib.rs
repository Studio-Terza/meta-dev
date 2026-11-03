mod emitters;
mod path_reach;
mod test_code;

use emitters::Emitters;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::{Headline, Lint};
use path_reach::PathReach;
use test_code::TestCode;

#[derive(Debug)]
pub struct PublicApiTests;

impl PublicApiTests {
    const FIX: &str = "test code in src; move it to the package's tests directory and call the public API from there";
    const REACH: &str = "test file includes a module from src with #[path]; call the public API instead";
}

impl Lint for PublicApiTests {
    fn report(files: &[SourceFile]) -> Report {
        let syntax: Vec<&syn::File> = files.iter().map(SourceFile::syntax).collect();
        let emitters = Emitters::defined_in(&syntax);
        Report::new(
            Headline::new("tests go through public APIs"),
            files
                .iter()
                .flat_map(|file| {
                    let (lines, message) = if file.is_shipped() {
                        (TestCode::lines_in(file.syntax(), &emitters), Self::FIX)
                    } else {
                        (PathReach::lines_in(file), Self::REACH)
                    };
                    lines.into_iter().map(move |line| Violation::new(Site::new(file.path(), line), message))
                })
                .collect(),
        )
    }
}
