mod guards;

use guards::{Guards, Scanned};
use lint_core::{Headline, Parsed, SourceFile, Violation};
use lint_file::{Concluded, ContextLint, Reads, RuledLint};
use lint_sweep::Folding;

#[derive(Debug)]
pub struct GuardMustUse;

impl ContextLint for GuardMustUse {
    const HEADLINE: Headline = Headline::new("guard types are #[must_use]");
    const READS: Reads = Reads::Shipped;

    type Part<'head> = Scanned<'head>;
    type Context<'head> = Concluded<'head>;

    type Gathering<'head> = Folding<Guards<'head>>;

    fn gathering<'head>(parsed: Parsed<'_, 'head>) -> Folding<Guards<'head>> {
        Folding::from(Guards::scanning(parsed))
    }
}

impl RuledLint for GuardMustUse {
    fn ruled<'file, 'head: 'file>(file: &'file SourceFile, concluded: &Concluded<'head>) -> Vec<Violation<'file>> {
        concluded.found_in(file)
    }
}
