mod module_tree;

use module_tree::ShippedModule;

use lint_core::{Bound, Headline, Parsed, SourceFile, Violation};
use lint_file::{Concluded, ContextLint, Reads};

#[derive(Debug)]
pub struct ModuleNesting;

impl ContextLint for ModuleNesting {
    const HEADLINE: Headline =
        Headline::new("modules nest by use: a file used only inside one sibling belongs inside it");
    const READS: Reads = Reads::Shipped;

    type Part<'head> = ShippedModule<'head>;
    type Context<'head> = Concluded<'head>;

    fn contributed<'head>(parsed: Parsed<'_, 'head>) -> ShippedModule<'head> {
        ShippedModule::from(parsed)
    }

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, concluded: &Concluded<'head>) -> Vec<Violation<'file>> {
        concluded.found_in(file)
    }
}

impl ModuleNesting {
    const REEXPORTS: Bound = Bound::Most(8);
}
