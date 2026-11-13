use std::path::Path;
use syn::spanned::Spanned;

use syn::{Index, ItemMod, ItemStruct, Visibility};

use super::finding::Finding;
use lint_core::Site;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_sweep::{Descend, Kind, Watch};

#[derive(Debug)]
pub struct FieldVisibility<'scan> {
    path: &'scan Path,
    violations: Vec<Violation<'scan>>,
}

impl<'scan> FieldVisibility<'scan> {
    pub fn new(path: &'scan Path) -> FieldVisibility<'scan> {
        FieldVisibility { path, violations: Vec::new() }
    }

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

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_struct(&mut self, item: &'ast ItemStruct) -> Descend {
        Descend::from(Walk::outside_tests(&item.attrs)).then(|| self.exposed(item))
    }
}

impl<'file> From<FieldVisibility<'file>> for Vec<Violation<'file>> {
    fn from(scan: FieldVisibility<'file>) -> Vec<Violation<'file>> {
        scan.violations
    }
}
