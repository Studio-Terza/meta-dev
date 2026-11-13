mod emitters;
mod path_reach;
mod test_code;

use emitters::Emitters;
use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::{Headline, Parsed};
use lint_file::{ContextLint, Reads};
use path_reach::PathReach;
use test_code::TestCode;

#[derive(Debug)]
pub struct PublicApiTests;

impl PublicApiTests {
    const IN_SOURCE: Message = Message::new(
        Problem::new("test code in src"),
        Fix::new("move it to the package's tests directory and call the public API from there"),
    );
    const REACH: Message = Message::new(
        Problem::new("test file includes a module from src with #[path]"),
        Fix::new("call the public API instead"),
    );
}

impl ContextLint for PublicApiTests {
    const HEADLINE: Headline = Headline::new("tests go through public APIs");
    const READS: Reads = Reads::Every;

    type Part<'head> = Emitters;
    type Context<'head> = Emitters;

    type Gathering<'head> = Emitters;

    fn gathering(_: Parsed<'_, '_>) -> Emitters {
        Emitters::default()
    }

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, emitters: &Emitters) -> Vec<Violation<'file>> {
        let (lines, message) = file.shipped().map_or_else(
            || (PathReach::lines_in(file), Self::REACH),
            |shipped| (TestCode::lines_in(shipped.syntax(), emitters.as_ref()), Self::IN_SOURCE),
        );
        lines.into_iter().map(|line| Violation::new(Site::new(file.head().path(), line), message)).collect()
    }
}
