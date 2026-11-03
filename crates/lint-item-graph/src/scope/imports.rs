use std::collections::BTreeSet;

use lint_layout::Name;
use proc_macro2::Ident;

use super::import::Import;

#[derive(Debug, Default)]
pub struct Imports<'file>(Vec<Import<'file>>);

impl<'file> Imports<'file> {
    pub fn add(&mut self, import: Option<Import<'file>>) {
        self.0.extend(import);
    }

    pub fn holds<Spelled: PartialEq<Ident>>(&self, name: &Spelled) -> Option<&Import<'file>> {
        self.0.iter().find_map(|import| import.names(name))
    }

    pub fn local_item<Spelled: PartialEq<Ident>>(
        &self,
        name: &Spelled,
        local_roots: &BTreeSet<Name>,
    ) -> Option<&Imports<'file>> {
        self.0
            .iter()
            .filter_map(|import| import.names(name))
            .all(|import| import.rooted_in(local_roots).is_some())
            .then_some(self)
    }

    pub fn original(&self, name: Name) -> Name {
        self.holds(&name).map(Import::original).map_or(name, Name::from)
    }
}
