mod item_head;

use syn::visit::{self, Visit};
use syn::{Field, Item, ItemImpl, ItemMod};

use item_head::ItemHead;
use lint_core::Walk;

use crate::flagged::Flagged;

#[derive(Debug, Default)]
pub(crate) struct GatedItems {
    found: Vec<Flagged>,
}

impl GatedItems {
    pub(crate) fn in_file(file: &syn::File) -> Vec<Flagged> {
        let mut items = GatedItems::default();
        items.visit_file(file);
        items.found
    }
}

impl<'ast> Visit<'ast> for GatedItems {
    fn visit_item(&mut self, item: &'ast Item) {
        self.found.extend(ItemHead::of_item(item).and_then(|head| head.flagged(None)));
        visit::visit_item(self, item);
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let gated = ItemHead::gate_of(&item.attrs);
        self.found
            .extend(item.items.iter().filter_map(ItemHead::of_member).filter_map(|head| head.flagged(gated.clone())));
        visit::visit_item_impl(self, item);
    }

    fn visit_field(&mut self, field: &'ast Field) {
        self.found.extend(ItemHead::of_field(field).flagged(None));
        visit::visit_field(self, field);
    }
}
