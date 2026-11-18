use std::path::Path;
use syntax::Spanned;

use syntax::{Index, ItemMod, ItemStruct, Visibility};

use super::finding::Finding;
use algebra::Semigroup;
use lint_core::Site;
use lint_core::Violation;
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_sweep::{Contribute, Contributed, Descend, Facts, Kind};

#[derive(Debug)]
pub struct FieldVisibility<'scan> {
    path: &'scan Path,
}

type Exposed<'scan> = Contributed<(), Vec<Violation<'scan>>>;

impl<'scan> FieldVisibility<'scan> {
    fn exposed(&self, item: &ItemStruct) -> Vec<Violation<'scan>> {
        item.fields
            .iter()
            .enumerate()
            .filter(|(_, field)| matches!(field.vis, Visibility::Public(_) | Visibility::Restricted(_)))
            .map(|(index, field)| {
                Violation::new(
                    Site::new(self.path, LineNumber::from(field.span())),
                    Finding::ExposedField { field, index: Index::from(index), owner: Name::from(&item.ident) },
                )
            })
            .collect()
    }
}

impl<'scan> Contribute for FieldVisibility<'scan> {
    type Open = ();
    type Part = Vec<Violation<'scan>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemStruct]
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Exposed<'scan> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_struct(
        &self,
        item: &ItemStruct,
        facts: &Facts<'_, ItemStruct>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Exposed<'scan> {
        let descend = Descend::from(facts.gate());
        let exposed = descend.entered().map_or_default(|()| self.exposed(item));
        Contributed::new((), exposed, descend.combine(Descend::Over))
    }
}

impl<'scan> From<&'scan Path> for FieldVisibility<'scan> {
    fn from(path: &'scan Path) -> FieldVisibility<'scan> {
        FieldVisibility { path }
    }
}
