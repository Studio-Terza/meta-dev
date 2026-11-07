use std::collections::{BTreeMap, BTreeSet};

use syn::{UseTree, Visibility};

use lint_core::SourceFile;
use lint_core::Syntax;
use lint_layout::Name;
use lint_placement::CrateName;
use lint_placement::Role;

#[derive(Default)]
pub struct RootExports(BTreeMap<CrateName, BTreeSet<Name>>);

impl RootExports {
    pub fn in_files(files: &[&SourceFile]) -> Self {
        let mut exports = Self::default();
        files.iter().filter(|file| file.placement().role() == Role::LibraryRoot).for_each(|file| exports.record(file));
        exports
    }

    pub fn exported(&self, file: &SourceFile, name: &Name) -> Option<&Name> {
        self.0.get(&CrateName::from(file.placement().crate_root()))?.get(name)
    }

    fn record(&mut self, file: &SourceFile) {
        let names = file
            .syntax()
            .items
            .iter()
            .filter_map(Syntax::item_use)
            .filter(|used| matches!(used.vis, Visibility::Public(_)))
            .flat_map(|used| Self::leaves(&used.tree));
        self.0.entry(CrateName::from(file.placement().crate_root())).or_default().extend(names);
    }

    fn leaves(tree: &UseTree) -> Vec<Name> {
        match tree {
            UseTree::Path(path) => Self::leaves(&path.tree),
            UseTree::Group(group) => group.items.iter().flat_map(Self::leaves).collect(),
            UseTree::Name(name) => vec![Name::from(&name.ident)],
            UseTree::Rename(rename) => vec![Name::from(&rename.rename)],
            UseTree::Glob(_) => Vec::new(),
        }
    }
}
