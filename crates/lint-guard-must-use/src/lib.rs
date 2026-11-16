mod guards;

use guards::Guards;
use guards::Modules;
use lint_core::{Headline, Parsed, SourceFile, Violation};
use lint_file::{Concluded, ContextLint, Reads};
use lint_sweep::Unwatched;

#[derive(Debug)]
pub struct GuardMustUse;

impl ContextLint for GuardMustUse {
    const HEADLINE: Headline = Headline::new("guard types are #[must_use]");
    const READS: Reads = Reads::Shipped;

    type Part<'head> = Modules<'head>;
    type Context<'head> = Concluded<'head>;

    type Gathering<'head> = Guards<'head>;

    fn gathering<'head>(parsed: Parsed<'_, 'head>) -> Guards<'head> {
        Guards::scanning(parsed)
    }

    type Found<'file> = Unwatched;
    type Watched<'file> = Unwatched;

    fn watched(_: &SourceFile) -> Unwatched {
        Unwatched
    }

    fn judged<'file, 'head: 'file>(
        file: &'file SourceFile,
        _: Unwatched,
        concluded: &Concluded<'head>,
    ) -> Vec<Violation<'file>> {
        concluded.found_in(file)
    }
}
