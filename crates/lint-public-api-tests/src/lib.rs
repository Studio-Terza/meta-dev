mod emitters;
mod path_reach;
mod test_code;

use emitters::Emitters;
use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::SourceFile;
use lint_core::{Headline, Parsed};
use lint_file::{ContextLint, Reads};
use lint_sweep::Judgement;
use path_reach::PathReach;
use test_code::TestCode;

#[derive(Debug)]
pub struct PublicApiTests;

impl PublicApiTests {
    pub(crate) const IN_SOURCE: Message = Message::new(
        Problem::new("test code in src"),
        Fix::new("move it to the package's tests directory and call the public API from there"),
    );
    pub(crate) const REACH: Message = Message::new(
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

    fn judgement<'run, 'file: 'run, 'head: 'file>(
        file: &'file SourceFile,
        emitters: &'run Emitters,
    ) -> Judgement<'file, 'run> {
        file.shipped().map_or_else(
            || Judgement::watching(Box::new(PathReach::beside(file))),
            |_| Judgement::watching(Box::new(TestCode::new(file.head().path(), emitters.as_ref()))),
        )
    }
}
