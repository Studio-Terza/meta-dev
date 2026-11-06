use std::collections::BTreeSet;

use lint_layout::Name;

use super::import::Import;

#[derive(Debug, Default)]
pub struct Imports<'file>(Vec<Import<'file>>);

impl<'file> Imports<'file> {
    pub fn add(&mut self, import: Option<Import<'file>>) {
        self.0.extend(import);
    }

    pub fn holds(&self, name: &Name) -> Option<&Import<'file>> {
        self.0.iter().find_map(|import| import.names(name))
    }

    pub fn path_of(&self, name: &Name) -> Option<Vec<Name>> {
        self.0.iter().find_map(|import| import.names(name)).map(Import::spelled_path)
    }

    pub fn local_item(&self, name: &Name, local_roots: &BTreeSet<Name>) -> Option<&Imports<'file>> {
        self.0
            .iter()
            .filter_map(|import| import.names(name))
            .all(|import| import.rooted_in(local_roots).is_some())
            .then_some(self)
    }
}
