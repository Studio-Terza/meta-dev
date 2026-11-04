use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use law_facts::{Kind, Law, Laws, Package, Property};
use lint_core::{Site, SourceFile, Violation};
use lint_layout::Name;

use crate::algebra::Algebras;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Checked {
    Operations,
    Preservation,
}

impl Checked {
    fn reporting(kind: Kind) -> Checked {
        match kind.property() {
            Some(Property::Preserves) => Checked::Preservation,
            Some(_) | None => Checked::Operations,
        }
    }
}

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

    pub(crate) fn of_package(&self, path: &Path) -> Vec<&Law<'found>> {
        self.packages.get(path).map(|package| self.laws.of_package(*package)).unwrap_or_default()
    }

    pub(crate) fn unresolved(&self, algebras: &Algebras<'_, '_>, checked: Checked) -> Vec<Violation<'found>> {
        let packages: BTreeSet<Package<'found>> = algebras
            .operations()
            .filter_map(|operation| self.packages.get(AsRef::<Site<'_>>::as_ref(operation).path()))
            .copied()
            .collect();
        self.laws
            .all()
            .filter(|law| Checked::reporting(law.kind()) == checked)
            .filter(|law| packages.iter().any(|package| law.of_package(*package).is_some()))
            .filter_map(Law::unresolved)
            .map(|unresolved| Violation::new(*AsRef::<Site<'found>>::as_ref(&unresolved), &unresolved))
            .collect()
    }
}
