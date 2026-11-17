mod item_head;

use syntax::{Field, Item, ItemImpl, ItemMod};

use item_head::ItemHead;
use std::path::Path;

use lint_core::Violation;
use lint_sweep::{Concludes, Contribute, Contributed, Descend, Facts, Kind};

use crate::flagged::Flagged;

#[derive(Debug)]
pub(crate) struct GatedItems<'file> {
    path: &'file Path,
}

impl Contribute for GatedItems<'_> {
    type Open = ();
    type Part = Vec<Flagged>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Item, Kind::ItemMod, Kind::ItemImpl, Kind::Field]
    }

    fn on_item(&self, item: &Item, _: &Facts<'_, Item>) -> Contributed<(), Vec<Flagged>> {
        Contributed::parted(ItemHead::of_item(item).and_then(|head| head.flagged(None)).into_iter().collect())
    }

    fn on_item_mod(&self, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Contributed<(), Vec<Flagged>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_impl(&self, item: &ItemImpl, _: &Facts<'_, ItemImpl>) -> Contributed<(), Vec<Flagged>> {
        let gated = ItemHead::gate_of(&item.attrs);
        Contributed::parted(item.items.iter().filter_map(ItemHead::of_member).filter_map(|head| head.flagged(gated)).collect())
    }

    fn on_field(&self, field: &Field, _: &Facts<'_, Field>) -> Contributed<(), Vec<Flagged>> {
        Contributed::parted(ItemHead::of_field(field).flagged(None).into_iter().collect())
    }
}

impl<'file> Concludes<'file> for GatedItems<'file> {
    fn violations(self, (): (), found: Vec<Flagged>) -> Vec<Violation<'file>> {
        found.into_iter().map(|flagged| flagged.in_file(self.path)).collect()
    }
}

impl<'file> From<&'file Path> for GatedItems<'file> {
    fn from(path: &'file Path) -> GatedItems<'file> {
        GatedItems { path }
    }
}
