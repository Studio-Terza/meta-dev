mod item_head;

use syntax::{Field, Item, ItemImpl, ItemMod};

use item_head::ItemHead;
use std::path::Path;

use lint_core::Violation;
use lint_sweep::{Descend, Facts, Finds, Kind, Watch};

use crate::flagged::Flagged;

#[derive(Debug)]
pub(crate) struct GatedItems<'file> {
    path: &'file Path,
    found: Vec<Flagged>,
}

impl<'file> GatedItems<'file> {
    pub(crate) fn new(path: &'file Path) -> GatedItems<'file> {
        GatedItems { path, found: Vec::new() }
    }
}

impl<'ast> Watch<'ast> for GatedItems<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Item, Kind::ItemMod, Kind::ItemImpl, Kind::Field]
    }

    fn on_item(&mut self, item: &'ast Item, _: &Facts<'_, Item>) -> Descend {
        self.found.extend(ItemHead::of_item(item).and_then(|head| head.flagged(None)));
        Descend::Into
    }

    fn on_item_mod(&mut self, _: &'ast ItemMod, facts: &Facts<'_, ItemMod>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl, _: &Facts<'_, ItemImpl>) -> Descend {
        let gated = ItemHead::gate_of(&item.attrs);
        self.found.extend(item.items.iter().filter_map(ItemHead::of_member).filter_map(|head| head.flagged(gated)));
        Descend::Into
    }

    fn on_field(&mut self, field: &'ast Field, _: &Facts<'_, Field>) -> Descend {
        self.found.extend(ItemHead::of_field(field).flagged(None));
        Descend::Into
    }
}

impl<'file> Finds<'file> for GatedItems<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        let path = self.path;
        self.found.into_iter().map(|flagged| flagged.in_file(path)).collect()
    }
}
