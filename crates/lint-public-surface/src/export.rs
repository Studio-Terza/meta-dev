use syn::{Item, ItemMod, ItemUse, UseTree, Visibility};

use lint_core::SourceFile;
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_placement::Role;

#[derive(Debug, PartialEq, Eq)]
pub struct Export {
    name: Name,
    line: LineNumber,
}

impl Export {
    pub fn in_root(file: &SourceFile) -> Vec<Export> {
        (file.placement().role() == Role::LibraryRoot).then_some(file).map_or_else(Vec::new, |file| {
            Self::reexports(file)
                .flat_map(|used| Self::leaves(&used.tree))
                .chain(Self::modules(file).map(Self::of_module))
                .collect()
        })
    }

    fn reexports(file: &SourceFile) -> impl Iterator<Item = &ItemUse> {
        file.syntax().items.iter().filter_map(|item| match item {
            Item::Use(used) if matches!(used.vis, Visibility::Public(_)) => Some(used),
            _ => None,
        })
    }

    fn modules(file: &SourceFile) -> impl Iterator<Item = &ItemMod> {
        file.syntax().items.iter().filter_map(|item| match item {
            Item::Mod(module) if matches!(module.vis, Visibility::Public(_)) => Some(module),
            _ => None,
        })
    }

    fn of_module(module: &ItemMod) -> Export {
        Export { name: Name::from(&module.ident), line: LineNumber::from(module.ident.span()) }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn line(&self) -> LineNumber {
        self.line
    }

    fn leaves(tree: &UseTree) -> Vec<Export> {
        match tree {
            UseTree::Path(path) => Self::leaves(&path.tree),
            UseTree::Group(group) => group.items.iter().flat_map(Self::leaves).collect(),
            UseTree::Name(name) => {
                vec![Export { name: Name::from(&name.ident), line: LineNumber::from(name.ident.span()) }]
            }
            UseTree::Rename(rename) => {
                vec![Export { name: Name::from(&rename.rename), line: LineNumber::from(rename.rename.span()) }]
            }
            UseTree::Glob(_) => Vec::new(),
        }
    }
}
