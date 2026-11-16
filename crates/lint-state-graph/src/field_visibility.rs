use std::path::Path;
use syntax::Spanned;

use syntax::{Index, ItemMod, ItemStruct, Visibility};

use super::finding::Finding;
use lint_core::Site;
use lint_core::Violation;
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_sweep::{Descend, Facts, Kind, Watch};

#[derive(Debug)]
pub struct FieldVisibility<'scan> {
    path: &'scan Path,
    violations: Vec<Violation<'scan>>,
}

impl<'scan> FieldVisibility<'scan> {
    fn exposed(&mut self, item: &ItemStruct) -> Descend {
        let exposed = item
            .fields
            .iter()
            .enumerate()
            .filter(|(_, field)| matches!(field.vis, Visibility::Public(_) | Visibility::Restricted(_)))
            .map(|(index, field)| {
                Violation::new(
                    Site::new(self.path, LineNumber::from(field.span())),
                    Finding::ExposedField { field, index: Index::from(index), owner: Name::from(&item.ident) },
                )
            });
        self.violations.extend(exposed);
        Descend::Over
    }
}

impl<'ast> Watch<'ast> for FieldVisibility<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemStruct]
    }

    fn on_item_mod(&mut self, _: &'ast ItemMod, facts: &Facts<'_, ItemMod>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_item_struct(&mut self, item: &'ast ItemStruct, facts: &Facts<'_, ItemStruct>) -> Descend {
        Descend::from(facts.gate()).then(|| self.exposed(item))
    }
}

impl<'file> From<FieldVisibility<'file>> for Vec<Violation<'file>> {
    fn from(scan: FieldVisibility<'file>) -> Vec<Violation<'file>> {
        scan.violations
    }
}

impl<'scan> From<&'scan Path> for FieldVisibility<'scan> {
    fn from(path: &'scan Path) -> FieldVisibility<'scan> {
        FieldVisibility { path, violations: Vec::new() }
    }
}
