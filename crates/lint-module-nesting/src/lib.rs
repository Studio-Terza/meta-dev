mod module_tree;

use module_tree::{ModuleGathering, ShippedModule};

use lint_core::{Bound, Headline, Parsed, SourceFile};
use lint_file::{Concluded, ContextLint, Reads};
use lint_sweep::Judgement;

#[derive(Debug)]
pub struct ModuleNesting;

impl ContextLint for ModuleNesting {
    const HEADLINE: Headline =
        Headline::new("modules nest by use: a file used only inside one sibling belongs inside it");
    const READS: Reads = Reads::Shipped;

    type Part<'head> = ShippedModule<'head>;
    type Context<'head> = Concluded<'head>;

    type Gathering<'head> = ModuleGathering<'head>;

    fn gathering<'head>(parsed: Parsed<'_, 'head>) -> ModuleGathering<'head> {
        ModuleGathering::from(parsed)
    }

    fn judgement<'run, 'file: 'run, 'head: 'file>(
        file: &'file SourceFile,
        concluded: &'run Concluded<'head>,
    ) -> Judgement<'file, 'run> {
        Judgement::found(concluded.found_in(file))
    }
}

impl ModuleNesting {
    const REEXPORTS: Bound = Bound::Most(8);
}
