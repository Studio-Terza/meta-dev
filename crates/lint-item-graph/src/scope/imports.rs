use std::collections::BTreeSet;

use lint_layout::{Name, UsePath};

use super::import::Import;

#[derive(Debug, Default)]
pub struct Imports(Vec<Import>);

impl Imports {
    pub fn add(&mut self, import: Import) {
        self.0.push(import);
    }

    pub fn holds(&self, name: &Name) -> bool {
        self.0.iter().any(|import| import.names(name).is_some())
    }

    pub fn local_item(&self, name: &Name, local_roots: &BTreeSet<Name>) -> Option<Name> {
        let paths: Vec<&UsePath> = self.0.iter().filter_map(|import| import.names(name)).collect();
        let original = paths.iter().find_map(|path| path.segments().last());
        paths
            .iter()
            .all(|path| path.first().is_some_and(|root| local_roots.contains(root)))
            .then(|| original.unwrap_or(name).clone())
    }
}
