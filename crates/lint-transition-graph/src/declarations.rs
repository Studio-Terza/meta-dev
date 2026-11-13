use std::collections::BTreeSet;
use std::path::Path;

use graph::{LawKind, Model, Property};
use lint_core::{Site, Violation};

use crate::laws::{Law, Laws};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Checked {
    Operations,
    Preservation,
}

impl Checked {
    fn reporting(kind: LawKind) -> Checked {
        match kind.property() {
            Some(Property::Preserves) => Checked::Preservation,
            Some(_) | None => Checked::Operations,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Declarations<'file> {
    laws: Laws<'file>,
}

impl<'file> From<&'file Model> for Declarations<'file> {
    fn from(model: &'file Model) -> Declarations<'file> {
        Declarations { laws: Laws::from(model) }
    }
}

impl<'file> Declarations<'file> {
    pub(crate) fn laws(&self) -> &Laws<'file> {
        &self.laws
    }

    pub(crate) fn unresolved<'site>(
        &self,
        sites: impl Iterator<Item = Site<'site>>,
        checked: Checked,
    ) -> Vec<Violation<'file>> {
        let paths: BTreeSet<&Path> = sites.map(|site| site.path()).collect();
        let listed: Vec<&Path> = paths.into_iter().collect();
        self.laws
            .of_crates(&listed)
            .filter(|law| Checked::reporting(law.kind()) == checked)
            .filter_map(Law::unresolved)
            .map(|unresolved| Violation::new(*AsRef::<Site<'file>>::as_ref(&unresolved), &unresolved))
            .collect()
    }
}
