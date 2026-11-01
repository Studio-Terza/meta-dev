use syn::{Item, ItemMod, ItemUse, UseTree, Visibility};

use lint_core::SourceFile;
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_placement::CrateName;
use lint_placement::Role;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Export {
    crate_name: CrateName,
    name: Name,
    line: LineNumber,
}

impl Export {
    pub fn in_root(file: &SourceFile) -> Vec<Export> {
        (file.placement().role() == Role::LibraryRoot)
            .then(|| CrateName::from(file.placement().crate_root()))
            .map_or_else(Vec::new, |crate_name| {
                Self::reexports(file)
                    .flat_map(|used| Self::leaves(&crate_name, &used.tree))
                    .chain(Self::modules(file).map(|module| Self::of_module(&crate_name, module)))
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

    fn of_module(crate_name: &CrateName, module: &ItemMod) -> Export {
        Export {
            crate_name: crate_name.clone(),
            name: Name::from(&module.ident),
            line: LineNumber::from(module.ident.span()),
        }
    }

    pub fn crate_name(&self) -> &CrateName {
        &self.crate_name
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn line(&self) -> LineNumber {
        self.line
    }

    fn leaves(crate_name: &CrateName, tree: &UseTree) -> Vec<Export> {
        match tree {
            UseTree::Path(path) => Self::leaves(crate_name, &path.tree),
            UseTree::Group(group) => group.items.iter().flat_map(|item| Self::leaves(crate_name, item)).collect(),
            UseTree::Name(name) => vec![Export {
                crate_name: crate_name.clone(),
                name: Name::from(&name.ident),
                line: LineNumber::from(name.ident.span()),
            }],
            UseTree::Rename(rename) => vec![Export {
                crate_name: crate_name.clone(),
                name: Name::from(&rename.rename),
                line: LineNumber::from(rename.rename.span()),
            }],
            UseTree::Glob(_) => Vec::new(),
        }
    }
}
