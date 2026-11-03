mod finding;
mod members;
mod shape;

use std::collections::BTreeMap;

use syn::visit::Visit;
use syn::{ImplItemFn, ItemImpl, ItemMod, Macro};

use lint_core::Macros;
use lint_core::Syntax;
use lint_core::Walk;

use finding::Finding;
use lint_core::{Site, Violation};
use members::Members;
use shape::Shape;
use std::path::Path;

#[derive(Default)]
pub struct SignatureIndex(BTreeMap<Shape, Members>);

impl SignatureIndex {
    pub fn violations(self, path: &Path) -> impl Iterator<Item = Violation<'_>> {
        self.0.into_iter().filter(|(_, members)| members.names().len() > 1).map(move |(shape, members)| {
            Violation::new(Site::new(path, members.first()), Finding::new(members.names(), shape))
        })
    }

    fn indexed(&mut self, item: &ItemImpl) {
        match Walk::inherent_impls(item) {
            Walk::Entered => Self::functions(item).for_each(|function| self.noted(item, function)),
            Walk::Skipped => {}
        }
    }

    fn noted(&mut self, item: &ItemImpl, function: &ImplItemFn) {
        let name = &function.sig.ident;
        self.0.entry(Shape::describe(item, function)).or_insert_with(|| Members::starting(name)).joined(name);
    }

    fn functions(item: &ItemImpl) -> impl Iterator<Item = &ImplItemFn> {
        item.items.iter().filter_map(Syntax::impl_fn)
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
