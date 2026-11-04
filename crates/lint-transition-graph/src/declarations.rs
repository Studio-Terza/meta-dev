use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use law_facts::{Law, Laws, Package, Property};
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

impl Declarations<'_> {
    pub(crate) fn properties_naming(&self, path: &Path, names: &[&Name]) -> BTreeSet<Property> {
        self.packages.get(path).map(|package| self.laws.properties_naming(*package, names)).unwrap_or_default()
    }

    pub(crate) fn laws_naming(&self, path: &Path, names: &[&Name]) -> Vec<&Law<'_>> {
        self.packages.get(path).map(|package| self.laws.naming(*package, names)).unwrap_or_default()
    }
}
