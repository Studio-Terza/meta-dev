use syntax::{ItemMod, ItemUse, UseLeaf, UseTree, Visibility};

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
    pub fn in_root(file: &SourceFile) -> Vec<Export> {
        (file.head().placement().role() == Role::LibraryRoot).then_some(file).map_or_else(Vec::new, |root| {
            Self::reexports(root)
                .flat_map(|used| Self::leaves(&used.tree))
                .chain(Self::modules(root).map(Self::of_module))
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
        let bound = UseLeaf::of_tree(tree).into_iter().filter_map(|leaf| leaf.end().bound());
        bound.map(|ident| Export { name: Name::from(ident), line: LineNumber::from(ident.span()) }).collect()
    }
}
