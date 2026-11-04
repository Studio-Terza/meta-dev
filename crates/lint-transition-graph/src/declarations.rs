use std::collections::BTreeMap;
use std::path::Path;

use law_facts::{Law, Laws, Package, Unresolved};
use lint_core::SourceFile;
use lint_layout::Name;

#[derive(Debug)]
pub(crate) struct Declarations<'found> {
    laws: Laws<'found>,
    packages: BTreeMap<&'found Path, Package<'found>>,
}

impl<'found> From<&'found [SourceFile]> for Declarations<'found> {
    fn from(files: &'found [SourceFile]) -> Declarations<'found> {
        Declarations {
            laws: Laws::from(files),
            packages: files.iter().map(|file| (file.path(), Package::containing(file))).collect(),
        }
    }
}

impl<'found> Declarations<'found> {
    pub(crate) fn routing(&self, path: &Path, owner: &Name, member: &Name) -> Vec<&Law<'found>> {
        self.packages.get(path).map(|package| self.laws.routing(*package, owner, member)).unwrap_or_default()
    }

    pub(crate) fn unresolved(&self, path: &Path) -> Vec<Unresolved<'found>> {
        self.packages.get(path).map(|package| self.laws.unresolved(*package)).unwrap_or_default()
    }
}
