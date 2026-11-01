use std::collections::{BTreeMap, BTreeSet};

use syn::{Item, UseTree, Visibility};

use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateName;
use lint_placement::Role;

#[derive(Default)]
pub struct RootExports(BTreeMap<CrateName, BTreeSet<Name>>);

impl RootExports {
    pub fn in_files(files: &[&SourceFile]) -> RootExports {
        let mut exports = RootExports::default();
        files.iter().filter(|file| file.placement().role() == Role::LibraryRoot).for_each(|file| exports.record(file));
        exports
    }

    pub fn contains(&self, file: &SourceFile, name: &Name) -> bool {
        self.0.get(&CrateName::from(file.placement().crate_root())).is_some_and(|names| names.contains(name))
    }

    fn record(&mut self, file: &SourceFile) {
        let names = file.syntax().items.iter().flat_map(|item| match item {
            Item::Use(used) if matches!(used.vis, Visibility::Public(_)) => Self::leaves(&used.tree),
            _ => Vec::new(),
        });
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
