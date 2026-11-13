mod guards;

use guards::Guards;
pub use guards::Modules;
use lint_core::{Headline, Parsed, SourceFile, Violation};
use lint_file::{Concluded, ContextLint, Reads};

#[derive(Debug)]
pub struct GuardMustUse;

impl ContextLint for GuardMustUse {
    const HEADLINE: Headline = Headline::new("guard types are #[must_use]");
    const READS: Reads = Reads::Shipped;

    type Part<'head> = Modules<'head>;
    type Context<'head> = Concluded<'head>;

    fn contributed<'head>(parsed: Parsed<'_, 'head>) -> Modules<'head> {
        parsed.file().shipped().map_or_default(|_| Guards::scanned(parsed))
    }

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, concluded: &Concluded<'head>) -> Vec<Violation<'file>> {
        concluded.of(file)
    }
}
