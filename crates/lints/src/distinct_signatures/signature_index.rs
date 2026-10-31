use std::collections::BTreeMap;

use syn::visit::Visit;
use syn::{Ident, ImplItem, ImplItemFn, ItemImpl, ItemMod};

use lint_core::Walk;

use super::shape::Shape;

#[derive(Default)]
pub struct SignatureIndex(BTreeMap<Shape, Vec<Ident>>);

impl SignatureIndex {
    pub fn shared(self) -> BTreeMap<Shape, Vec<Ident>> {
        self.0.into_iter().filter(|(_, names)| names.len() > 1).collect()
    }

    fn functions(item: &ItemImpl) -> impl Iterator<Item = &ImplItemFn> {
        item.items.iter().filter_map(|member| match member {
            ImplItem::Fn(function) => Some(function),
            _ => None,
        })
    }
}

impl<'ast> Visit<'ast> for SignatureIndex {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        match Walk::inherent_impls(item) {
            Walk::Entered => Self::functions(item).for_each(|function| {
                self.0.entry(Shape::describe(item, &function.sig)).or_default().push(function.sig.ident.clone());
            }),
            Walk::Skipped => {}
        }
    }
}
