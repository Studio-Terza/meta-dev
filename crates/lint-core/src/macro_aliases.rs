use std::collections::BTreeSet;

use syntax::{ItemUse, UseLeaf, UseTree};

use crate::watch::{Descend, Facts, Kind, Watch};

use lint_layout::Name;
use lint_layout::Word;

#[derive(Debug)]
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

    fn renamed(&self, tree: &UseTree) -> Vec<Name> {
        let renames = UseLeaf::of_tree(tree).into_iter().filter_map(|leaf| leaf.end().renamed());
        renames.filter(|renamed| renamed.ident == self.original).map(|renamed| Name::from(&renamed.rename)).collect()
    }
}

impl<'ast> Watch<'ast> for MacroAliases {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemUse]
    }

    fn on_item_use(&mut self, item: &'ast ItemUse, _: &Facts<'_, ItemUse>) -> Descend {
        let renamed = self.renamed(&item.tree);
        self.names.extend(renamed);
        Descend::Over
    }
}
