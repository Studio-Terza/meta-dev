mod module_tree;

use module_tree::{ModuleGathering, ShippedModule};

use lint_core::{Bound, Headline, Parsed, SourceFile, Violation};
use lint_file::{Concluded, ContextLint, Reads};
use lint_sweep::Unwatched;

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

impl ModuleNesting {
    const REEXPORTS: Bound = Bound::Most(8);
}
