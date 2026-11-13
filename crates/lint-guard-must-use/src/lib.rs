mod guards;

use guards::Guards;
use guards::Modules;
use lint_core::{Headline, Parsed, SourceFile};
use lint_file::{Concluded, ContextLint, Reads};
use lint_sweep::Judgement;

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

    fn judgement<'run, 'file: 'run, 'head: 'file>(
        file: &'file SourceFile,
        concluded: &'run Concluded<'head>,
    ) -> Judgement<'file, 'run> {
        Judgement::found(concluded.found_in(file))
    }
}
