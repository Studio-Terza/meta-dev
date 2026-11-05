mod module_tree;

use lint_core::Lint;
use module_tree::ModuleTree;

use lint_core::SourceFile;
use lint_core::{Headline, Report};

#[derive(Debug)]
pub struct ModuleNesting;

impl Lint for ModuleNesting {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let shipped: Vec<&SourceFile> = files.iter().filter_map(SourceFile::shipped).collect();
        let tree = ModuleTree::from(shipped.as_slice());
        Report::new(
            Headline::new(
                "a module is reached through its parent: a use path does not reach past a directory module into its children",
            ),
            tree.violations().collect(),
        )
    }
}
