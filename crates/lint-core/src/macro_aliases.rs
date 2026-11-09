use std::collections::BTreeSet;

use syn::visit::Visit;
use syn::{File, ItemUse, UseRename, UseTree};

use lint_layout::Name;
use lint_layout::Word;

#[derive(Debug)]
pub struct MacroAliases {
    original: Name,
    names: BTreeSet<Name>,
}

impl MacroAliases {
    #[must_use]
    pub fn in_file(file: &File, original: Word<'_>) -> MacroAliases {
        let mut aliases = MacroAliases { names: BTreeSet::new(), original: Name::from(original) };
        aliases.visit_file(file);
        aliases
    }

    #[must_use]
    pub fn call(&self, name: &Name) -> Option<&Name> {
        Some(&self.original).filter(|original| *original == name).or_else(|| self.names.get(name))
    }

    fn renamed(&self, tree: &UseTree) -> Vec<Name> {
        match tree {
            UseTree::Path(path) => self.renamed(&path.tree),
            UseTree::Rename(rename) => self.alias(rename),
            UseTree::Group(group) => group.items.iter().flat_map(|item| self.renamed(item)).collect(),
            UseTree::Name(_) | UseTree::Glob(_) => Vec::new(),
        }
    }

    fn alias(&self, rename: &UseRename) -> Vec<Name> {
        Some(rename)
            .filter(|candidate| candidate.ident == self.original)
            .map(|matched| Name::from(&matched.rename))
            .into_iter()
            .collect()
    }
}

impl<'ast> Visit<'ast> for MacroAliases {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        let renamed = self.renamed(&item.tree);
        self.names.extend(renamed);
    }
}
