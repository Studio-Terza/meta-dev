use std::collections::BTreeSet;

use lint_layout::Name;
use proc_macro2::Ident;

#[derive(Clone, Debug)]
pub struct Import<'file> {
    leaf: &'file Ident,
    path: Vec<&'file Ident>,
    original: &'file Ident,
}

impl<'file> Import<'file> {
    const SELF_IMPORT: &'static str = "self";

    pub fn renamed(path: Vec<&'file Ident>, leaf: &'file Ident, original: &'file Ident) -> Import<'file> {
        Import { leaf, path, original }
    }

    pub fn named(path: Vec<&'file Ident>, ident: &'file Ident) -> Option<Import<'file>> {
        match (*ident == Self::SELF_IMPORT).then_some(()) {
            Some(()) => Import::itself(path),
            None => Some(Import::renamed(path, ident, ident)),
        }
    }

    pub fn itself(mut path: Vec<&'file Ident>) -> Option<Import<'file>> {
        path.pop().map(|leaf| Import { leaf, path, original: leaf })
    }

    pub fn names<Spelled: PartialEq<Ident>>(&self, name: &Spelled) -> Option<&Import<'file>> {
        (*name == *self.leaf).then_some(self)
    }

    pub fn rooted_in(&self, local_roots: &BTreeSet<Name>) -> Option<&Import<'file>> {
        let root = self.path.first().copied().unwrap_or(self.original);
        local_roots.iter().any(|local| *local == *root).then_some(self)
    }

    pub fn original(&self) -> &'file Ident {
        self.original
    }

    pub fn spelled_path(&self) -> Vec<Name> {
        self.path.iter().copied().chain([self.original]).map(Name::from).collect()
    }
}
