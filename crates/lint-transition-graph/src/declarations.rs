use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use law_facts::{Kind, Law, Laws, Package, Property};
use lint_core::{Site, Violation};
use lint_layout::Name;

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
pub(crate) struct Declarations<'found, 'file> {
    laws: &'found Laws<'file>,
    packages: BTreeMap<&'file Path, Package<'file>>,
}

impl<'found, 'file> Declarations<'found, 'file> {
    pub(crate) fn new(
        laws: &'found Laws<'file>,
        packages: &BTreeMap<&'file Path, Package<'file>>,
    ) -> Declarations<'found, 'file> {
        Declarations { laws, packages: packages.clone() }
    }

    pub(crate) fn routing(&self, path: &Path, owner: &Name, member: &Name) -> Vec<&Law<'file>> {
        self.packages.get(path).map_or_default(|package| self.laws.routing(*package, owner, member))
    }

    pub(crate) fn of_package(&self, path: &Path) -> Vec<&Law<'file>> {
        self.packages.get(path).map_or_default(|package| self.laws.of_package(*package))
    }

    pub(crate) fn unresolved<'site>(
        &self,
        sites: impl Iterator<Item = Site<'site>>,
        checked: Checked,
    ) -> Vec<Violation<'file>> {
        let packages: BTreeSet<Package<'file>> =
            sites.filter_map(|site| self.packages.get(site.path())).copied().collect();
        self.laws
            .all()
            .filter(|law| Checked::reporting(law.kind()) == checked)
            .filter(|law| packages.iter().any(|package| law.of_package(*package).is_some()))
            .filter_map(Law::unresolved)
            .map(|unresolved| Violation::new(*AsRef::<Site<'file>>::as_ref(&unresolved), &unresolved))
            .collect()
    }
}
