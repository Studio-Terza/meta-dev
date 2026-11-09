use std::collections::{BTreeMap, BTreeSet};
use std::mem;

use lint_layout::Name;

use super::import::{Import, Leaf};

#[derive(Debug, Default)]
pub struct Imports<'file>(BTreeMap<Leaf, Vec<Import<'file>>>);

impl<'file> Imports<'file> {
    pub fn add(&mut self, import: Option<Import<'file>>) {
        self.0 = import.into_iter().fold(mem::take(&mut self.0), |mut grouped, found| {
            grouped.entry(Leaf::from(&found)).or_default().push(found);
            grouped
        });
    }

    pub fn path_of(&self, name: &Name) -> Option<Vec<Name>> {
        self.naming(name).first().map(Import::spelled_path)
    }

    pub fn local_item(&self, name: &Name, local_roots: &BTreeSet<Name>) -> Option<&Imports<'file>> {
        self.naming(name).iter().all(|import| import.rooted_in(local_roots).is_some()).then_some(self)
    }

    pub fn naming(&self, name: &Name) -> &[Import<'file>] {
        self.0.get(name).map_or(&[], Vec::as_slice)
    }
}
