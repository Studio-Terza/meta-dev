mod module_tree;

use lint_core::Lint;
use module_tree::ModuleTree;

use lint_core::SourceFile;
use lint_core::{Headline, Most, Report};

#[derive(Debug)]
pub struct ModuleNesting;

impl Lint for ModuleNesting {
    fn report(files: &[SourceFile]) -> Report {
        let shipped: Vec<&SourceFile> = files.iter().filter(|file| file.is_shipped()).collect();
        let tree = ModuleTree::from(shipped.as_slice());
        Report::new(
            Headline::new("modules nest by use: a file used only inside one sibling belongs inside it"),
            tree.violations().collect(),
        )
    }
}

impl ModuleNesting {
    const REEXPORTS: Most = Most::new(8);
}
