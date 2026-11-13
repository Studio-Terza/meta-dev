use std::borrow::Borrow;
use std::collections::BTreeSet;

use lint_layout::Name;
use proc_macro2::Ident;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Leaf(Name);

impl From<&Import> for Leaf {
    fn from(import: &Import) -> Leaf {
        Leaf(import.leaf.clone())
    }
}

impl Borrow<Name> for Leaf {
    fn borrow(&self) -> &Name {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct Import {
    leaf: Name,
    path: Vec<Name>,
    original: Name,
}

impl Import {
    const SELF_IMPORT: &'static str = "self";

    pub fn renamed(path: &[&Ident], leaf: &Ident, original: &Ident) -> Import {
        Import {
            leaf: Name::from(leaf),
            path: path.iter().copied().map(Name::from).collect(),
            original: Name::from(original),
        }
    }

    pub fn named(path: &[&Ident], ident: &Ident) -> Option<Import> {
        match (*ident == Self::SELF_IMPORT).then_some(()) {
            Some(()) => Import::itself(path),
            None => Some(Import::renamed(path, ident, ident)),
        }
    }

    pub fn itself(path: &[&Ident]) -> Option<Import> {
        let (leaf, rest) = path.split_last()?;
        Some(Import::renamed(rest, leaf, leaf))
    }

    pub fn rooted_in(&self, local_roots: &BTreeSet<Name>) -> Option<&Name> {
        let root = self.path.first().unwrap_or(&self.original);
        local_roots.get(root).map(|_| root)
    }

    pub fn original(&self) -> &Name {
        &self.original
    }

    pub fn spelled_path(&self) -> Vec<Name> {
        self.path.iter().chain([&self.original]).cloned().collect()
    }
}
