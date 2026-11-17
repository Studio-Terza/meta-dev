mod item_head;

use syntax::{Field, Item, ItemImpl, ItemMod};

use item_head::ItemHead;
use std::path::Path;

use lint_core::Violation;
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};

#[derive(Debug)]
pub(crate) struct GatedItems<'file> {
    path: &'file Path,
}

impl<'file> Fold for GatedItems<'file> {
    type Found = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Item, Kind::ItemMod, Kind::ItemImpl, Kind::Field]
    }

    fn on_item(&self, found: Self::Found, item: &Item, _: &Facts<'_, Item>) -> Visit<Self::Found> {
        Visit::adding(
            found,
            ItemHead::of_item(item).and_then(|head| head.flagged(None)).map(|flagged| flagged.in_file(self.path)),
            Descend::Into,
        )
    }

    fn on_item_mod(&self, found: Self::Found, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Self::Found> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_item_impl(&self, found: Self::Found, item: &ItemImpl, _: &Facts<'_, ItemImpl>) -> Visit<Self::Found> {
        let gated = ItemHead::gate_of(&item.attrs);
        let noted: Self::Found = found
            .into_iter()
            .chain(
                item.items
                    .iter()
                    .filter_map(ItemHead::of_member)
                    .filter_map(|head| head.flagged(gated))
                    .map(|flagged| flagged.in_file(self.path)),
            )
            .collect();
        Visit::new(noted, Descend::Into)
    }

    fn on_field(&self, found: Self::Found, field: &Field, _: &Facts<'_, Field>) -> Visit<Self::Found> {
        Visit::adding(
            found,
            ItemHead::of_field(field).flagged(None).map(|flagged| flagged.in_file(self.path)),
            Descend::Into,
        )
    }
}

impl<'file> From<&'file Path> for GatedItems<'file> {
    fn from(path: &'file Path) -> GatedItems<'file> {
        GatedItems { path }
    }
}
