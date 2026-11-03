mod signature_reading;

use syn::visit::{self, Visit};
use syn::{Ident, ImplItem, ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemTrait, TraitItem, TraitItemFn};

use lint_core::{Site, SourceFile, Walk};
use lint_layout::LineNumber;

use crate::context::Context;
use crate::ground::Ground;
use crate::origins::Origins;
use signature_reading::SignatureReading;

pub(super) struct Collector<'file> {
    file: &'file SourceFile,
    origins: Origins<'file>,
    ground: Ground,
    contexts: Vec<Context<'file>>,
}

impl<'file> Collector<'file> {
    pub(super) fn collected(file: &'file SourceFile) -> Vec<Context<'file>> {
        let mut scan = Collector {
            file,
            origins: Origins::of_file(file.placement().package().crate_name(), file.syntax()),
            ground: Ground::gated(file),
            contexts: Vec::new(),
        };
        scan.visit_file(file.syntax());
        scan.contexts
    }

    fn site(file: &'file SourceFile, name: &Ident) -> Site<'file> {
        Site::new(file.path(), LineNumber::from(name.span()))
    }

    fn trait_function(item: &TraitItem) -> Option<&TraitItemFn> {
        match item {
            TraitItem::Fn(function) => Some(function),
            _ => None,
        }
    }

    fn impl_function(item: &ImplItem) -> Option<&ImplItemFn> {
        match item {
            ImplItem::Fn(function) => Some(function),
            _ => None,
        }
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let outer = self.ground;
        self.ground = outer.within(module);
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
        self.ground = outer;
    }

    fn visit_item_trait(&mut self, found: &'ast ItemTrait) {
        let reading = SignatureReading::of_trait(&self.origins, found, self.ground);
        let file = self.file;
        let provided: Vec<_> = found
            .items
            .iter()
            .filter_map(Self::trait_function)
            .filter(|function| Walk::outside_tests(&function.attrs) == Walk::Entered)
            .filter_map(|function| {
                function
                    .default
                    .as_ref()
                    .map(|body| reading.function(Self::site(file, &function.sig.ident), &function.sig, body))
            })
            .collect();
        self.contexts.extend(
            (Walk::outside_tests(&found.attrs) == Walk::Entered)
                .then(|| reading.into_context().with_functions(provided)),
        );
    }

    fn visit_item_impl(&mut self, found: &'ast ItemImpl) {
        let reading = SignatureReading::of_impl(&self.origins, found, self.ground);
        let file = self.file;
        let functions: Vec<_> = found
            .items
            .iter()
            .filter_map(Self::impl_function)
            .filter(|function| Walk::outside_tests(&function.attrs) == Walk::Entered)
            .map(|function| reading.function(Self::site(file, &function.sig.ident), &function.sig, &function.block))
            .collect();
        self.contexts.extend(
            (Walk::outside_tests(&found.attrs) == Walk::Entered)
                .then(|| reading.into_context().with_functions(functions)),
        );
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        let reading = SignatureReading::free(&self.origins, self.ground);
        let found = reading.function(Self::site(self.file, &function.sig.ident), &function.sig, &function.block);
        self.contexts.extend(
            (Walk::outside_tests(&function.attrs) == Walk::Entered)
                .then(|| reading.into_context().with_functions([found])),
        );
    }
}
