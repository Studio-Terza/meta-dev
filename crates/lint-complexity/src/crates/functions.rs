use syn::visit::{self, Visit};
use syn::{Attribute, Block, Ident, ImplItemFn, ItemFn, ItemMod, TraitItemFn};

use body_lifts::{Branches, Propagation};
use lint_core::{Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name, TestMarks};

use super::function::Function;

pub(crate) struct Functions<'file> {
    file: &'file SourceFile,
    found: Vec<Function<'file>>,
}

impl<'file> Functions<'file> {
    pub(crate) fn of_file(file: &'file SourceFile) -> Vec<Function<'file>> {
        let mut scan = Functions { file, found: Vec::new() };
        scan.visit_file(file.syntax());
        scan.found
    }

    fn recorded(&mut self, attributes: &[Attribute], name: &Ident, body: Option<&Block>) {
        let function = |block: &Block| {
            Function::new(
                Name::from(name),
                Site::new(self.file.head().path(), LineNumber::from(name.span())),
                Branches::scored(block, Propagation::Branch),
            )
        };
        let found = body
            .filter(|_| Walk::outside_tests(attributes) == Walk::Entered)
            .filter(|_| TestMarks::mark_test(attributes).is_none())
            .map(function);
        self.found.extend(found);
    }
}

impl<'ast> Visit<'ast> for Functions<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.recorded(&function.attrs, &function.sig.ident, Some(&function.block));
        visit::visit_item_fn(self, function);
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.recorded(&function.attrs, &function.sig.ident, Some(&function.block));
        visit::visit_impl_item_fn(self, function);
    }

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        self.recorded(&function.attrs, &function.sig.ident, function.default.as_ref());
        visit::visit_trait_item_fn(self, function);
    }
}
