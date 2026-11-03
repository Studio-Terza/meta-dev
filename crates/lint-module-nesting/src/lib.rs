mod module_tree;

use lint_core::Lint;
use module_tree::ModuleTree;

use lint_core::SourceFile;
use lint_core::{Bound, Headline, Report};

#[derive(Debug)]
pub struct ModuleNesting;

impl Lint for ModuleNesting {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let shipped: Vec<&SourceFile> = files.iter().filter_map(SourceFile::shipped).collect();
        let tree = ModuleTree::from(shipped.as_slice());
        Report::new(
            Headline::new("modules nest by use: a file used only inside one sibling belongs inside it"),
            tree.violations().collect(),
        )
    }
}

impl ModuleNesting {
    const REEXPORTS: Bound = Bound::Most(8);
}
