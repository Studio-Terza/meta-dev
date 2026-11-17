use std::collections::BTreeSet;

use syntax::{ItemUse, UseLeaf, UseTree};

use lint_layout::Name;
use lint_layout::Word;

#[derive(Clone, Debug)]
pub struct MacroAliases {
    original: Name,
    names: BTreeSet<Name>,
}

impl MacroAliases {
    #[must_use]
    pub fn new(original: Word<'_>) -> MacroAliases {
        MacroAliases { names: BTreeSet::new(), original: Name::from(original) }
    }

    #[must_use]
    pub fn call(&self, name: &Name) -> Option<&Name> {
        Some(&self.original).filter(|original| *original == name).or_else(|| self.names.get(name))
    }

    #[must_use]
    pub fn used(self, item: &ItemUse) -> MacroAliases {
        let renamed = self.renamed(&item.tree);
        MacroAliases { names: self.names.into_iter().chain(renamed).collect(), ..self }
    }

    fn renamed(&self, tree: &UseTree) -> Vec<Name> {
        let renames = UseLeaf::of_tree(tree).into_iter().filter_map(|leaf| leaf.end().renamed());
        renames.filter(|renamed| renamed.ident == self.original).map(|renamed| Name::from(&renamed.rename)).collect()
    }
}
