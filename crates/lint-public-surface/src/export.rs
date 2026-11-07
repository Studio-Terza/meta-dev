use syn::{ItemMod, ItemUse, UseTree, Visibility};

use lint_core::{SourceFile, Syntax};
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_placement::Role;

#[derive(Debug, PartialEq, Eq)]
pub struct Export {
    name: Name,
    line: LineNumber,
}

impl Export {
    pub fn in_root(file: &SourceFile) -> Vec<Self> {
        (file.placement().role() == Role::LibraryRoot).then_some(file).map_or_else(Vec::new, |file| {
            Self::reexports(file)
                .flat_map(|used| Self::leaves(&used.tree))
                .chain(Self::modules(file).map(Self::of_module))
                .collect()
        })
    }

    fn reexports(file: &SourceFile) -> impl Iterator<Item = &ItemUse> {
        file.syntax().items.iter().filter_map(Syntax::item_use).filter(|used| matches!(used.vis, Visibility::Public(_)))
    }

    fn modules(file: &SourceFile) -> impl Iterator<Item = &ItemMod> {
        file.syntax()
            .items
            .iter()
            .filter_map(Syntax::item_mod)
            .filter(|module| matches!(module.vis, Visibility::Public(_)))
    }

    fn of_module(module: &ItemMod) -> Self {
        Self { name: Name::from(&module.ident), line: LineNumber::from(module.ident.span()) }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn line(&self) -> LineNumber {
        self.line
    }

    fn leaves(tree: &UseTree) -> Vec<Self> {
        match tree {
            UseTree::Path(path) => Self::leaves(&path.tree),
            UseTree::Group(group) => group.items.iter().flat_map(Self::leaves).collect(),
            UseTree::Name(name) => {
                vec![Self { name: Name::from(&name.ident), line: LineNumber::from(name.ident.span()) }]
            }
            UseTree::Rename(rename) => {
                vec![Self { name: Name::from(&rename.rename), line: LineNumber::from(rename.rename.span()) }]
            }
            UseTree::Glob(_) => Vec::new(),
        }
    }
}
