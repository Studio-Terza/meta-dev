use std::path::Path;
use syntax::Spanned;

use syntax::{Index, ItemMod, ItemStruct, Visibility};

use super::finding::Finding;
use lint_core::Site;
use lint_core::Violation;
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};

#[derive(Debug)]
pub struct FieldVisibility<'scan> {
    path: &'scan Path,
}

impl<'scan> FieldVisibility<'scan> {
    fn exposed(&self, found: Vec<Violation<'scan>>, item: &ItemStruct) -> Visit<Vec<Violation<'scan>>> {
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
        Visit::adding(found, exposed, Descend::Over)
    }
}

impl<'file> Fold for FieldVisibility<'file> {
    type Found = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemStruct]
    }

    fn on_item_mod(&self, found: Self::Found, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Self::Found> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_item_struct(
        &self,
        found: Self::Found,
        item: &ItemStruct,
        facts: &Facts<'_, ItemStruct>,
    ) -> Visit<Self::Found> {
        Visit::gated(found, Descend::from(facts.gate()), |open| self.exposed(open, item))
    }
}

impl<'scan> From<&'scan Path> for FieldVisibility<'scan> {
    fn from(path: &'scan Path) -> FieldVisibility<'scan> {
        FieldVisibility { path }
    }
}
