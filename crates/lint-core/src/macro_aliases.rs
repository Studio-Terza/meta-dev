use std::collections::BTreeSet;

use syn::visit::Visit;
use syn::{File, ItemUse, UseTree};

use lint_layout::Name;

#[derive(Debug)]
pub struct MacroAliases {
    original: Name,
    names: BTreeSet<Name>,
}

impl MacroAliases {
    #[must_use]
    pub fn in_file(file: &File, original: &str) -> MacroAliases {
        let original = Name::from(original);
        let mut aliases = MacroAliases { names: BTreeSet::from([original.clone()]), original };
        aliases.visit_file(file);
        aliases
    }

    #[must_use]
    pub fn call(&self, name: &Name) -> bool {
        self.names.contains(name)
    }

    fn renamed(&self, tree: &UseTree) -> Vec<Name> {
        match tree {
            UseTree::Path(path) => self.renamed(&path.tree),
            UseTree::Rename(rename) if self.original == rename.ident => vec![Name::from(&rename.rename)],
            UseTree::Group(group) => group.items.iter().flat_map(|item| self.renamed(item)).collect(),
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for MacroAliases {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        let renamed = self.renamed(&item.tree);
        self.names.extend(renamed);
    }
}
