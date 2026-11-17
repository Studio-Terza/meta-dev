use std::collections::BTreeSet;

use syntax::{UseLeaf, UseTree};

use algebra::{Monoid, Semigroup};
use lint_layout::Name;
use lint_layout::Word;

#[derive(Debug)]
pub struct MacroAliases {
    original: Name,
}

#[derive(Debug, Default)]
pub struct Aliases(BTreeSet<Name>);

#[derive(Debug)]
pub struct Aliased {
    original: Name,
    names: BTreeSet<Name>,
}

impl Semigroup for Aliases {
    fn combine(self, other: Aliases) -> Aliases {
        Aliases(self.0.into_iter().chain(other.0).collect())
    }
}

impl Monoid for Aliases {}

impl MacroAliases {
    #[must_use]
    pub fn new(original: Word<'_>) -> MacroAliases {
        MacroAliases { original: Name::from(original) }
    }

    #[must_use]
    pub fn renamed(&self, tree: &UseTree) -> Aliases {
        let renames = UseLeaf::of_tree(tree).into_iter().filter_map(|leaf| leaf.end().renamed());
        Aliases(
            renames
                .filter(|renamed| renamed.ident == self.original)
                .map(|renamed| Name::from(&renamed.rename))
                .collect(),
        )
    }

    #[must_use]
    pub fn aliased(self, aliases: Aliases) -> Aliased {
        Aliased { original: self.original, names: aliases.0 }
    }
}

impl Aliased {
    #[must_use]
    pub fn call(&self, name: &Name) -> Option<&Name> {
        Some(&self.original).filter(|original| *original == name).or_else(|| self.names.get(name))
    }
}
