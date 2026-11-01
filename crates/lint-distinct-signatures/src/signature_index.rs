use std::collections::BTreeMap;

use syn::visit::Visit;
use syn::{Ident, ImplItem, ImplItemFn, ItemImpl, ItemMod, Macro};

use lint_core::Macros;
use lint_core::Walk;

use super::shape::Shape;

#[derive(Default)]
pub struct SignatureIndex(BTreeMap<Shape, Vec<Ident>>);

impl SignatureIndex {
    pub fn shared(self) -> BTreeMap<Shape, Vec<Ident>> {
        self.0.into_iter().filter(|(_, names)| names.len() > 1).collect()
    }

    fn indexed(&mut self, item: &ItemImpl) {
        match Walk::inherent_impls(item) {
            Walk::Entered => Self::functions(item).for_each(|function| {
                self.0.entry(Shape::describe(item, function)).or_default().push(function.sig.ident.clone());
            }),
            Walk::Skipped => {}
        }
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
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || self.indexed(item));
    }
}
