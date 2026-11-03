use std::collections::BTreeSet;

use lint_layout::Name;
use proc_macro2::Ident;

#[derive(Clone, Copy, Debug)]
pub struct Import<'file> {
    leaf: &'file Ident,
    root: &'file Ident,
    original: &'file Ident,
}

impl<'file> Import<'file> {
    const SELF_IMPORT: &'static str = "self";

    pub fn renamed(first: Option<&'file Ident>, leaf: &'file Ident, original: &'file Ident) -> Import<'file> {
        Import { leaf, root: first.unwrap_or(original), original }
    }

    pub fn of_name(
        first: Option<&'file Ident>,
        last: Option<&'file Ident>,
        name: &'file Ident,
    ) -> Option<Import<'file>> {
        Some(name)
            .filter(|name| *name == Self::SELF_IMPORT)
            .map_or_else(|| Some(Import::renamed(first, name, name)), |_| Import::itself(first, last))
    }

    pub fn itself(first: Option<&'file Ident>, last: Option<&'file Ident>) -> Option<Import<'file>> {
        first.zip(last).map(|(root, leaf)| Import { leaf, root, original: leaf })
    }

    pub fn names<Spelled: PartialEq<Ident>>(&self, name: &Spelled) -> Option<&Import<'file>> {
        (*name == *self.leaf).then_some(self)
    }

    pub fn rooted_in(&self, local_roots: &BTreeSet<Name>) -> Option<&Import<'file>> {
        local_roots.iter().any(|root| *root == *self.root).then_some(self)
    }

    pub fn original(&self) -> &'file Ident {
        self.original
    }
}
