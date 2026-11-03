mod origins;
mod signature_reading;

use syn::visit::{self, Visit};
use syn::{Ident, ImplItem, ItemFn, ItemImpl, ItemMod, ItemTrait, TraitItem};

use lint_core::{Site, SourceFile, Walk};
use lint_layout::LineNumber;
use lint_placement::CrateName;

use crate::ground::Ground;
use crate::transition::Transition;
use origins::Origins;
use signature_reading::SignatureReading;

pub(super) struct Collector<'file> {
    file: &'file SourceFile,
    origins: Origins,
    ground: Ground,
    transitions: Vec<Transition>,
}

impl<'file> Collector<'file> {
    pub(super) fn collected(file: &'file SourceFile) -> Vec<Transition> {
        let mut scan = Collector {
            file,
            origins: Origins::of_file(CrateName::from(file.placement().package().root()), file.syntax()),
            ground: Ground::gated(file),
            transitions: Vec::new(),
        };
        scan.visit_file(file.syntax());
        scan.transitions
    }

    fn site(&self, name: &Ident) -> Site {
        Site::new(self.file.path(), LineNumber::from(name.span()))
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
        let provided: Vec<Transition> = found
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Fn(function) if Walk::outside_tests(&function.attrs) == Walk::Entered => Some(function),
                _ => None,
            })
            .filter_map(|function| {
                function
                    .default
                    .as_ref()
                    .map(|body| reading.transition(self.site(&function.sig.ident), &function.sig, body))
            })
            .collect();
        self.transitions.extend(provided.into_iter().filter(|_| Walk::outside_tests(&found.attrs) == Walk::Entered));
    }

    fn visit_item_impl(&mut self, found: &'ast ItemImpl) {
        let reading = SignatureReading::of_impl(&self.origins, found, self.ground);
        let transitions: Vec<Transition> = found
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(function) if Walk::outside_tests(&function.attrs) == Walk::Entered => {
                    Some(reading.transition(self.site(&function.sig.ident), &function.sig, &function.block))
                }
                _ => None,
            })
            .collect();
        self.transitions.extend(transitions.into_iter().filter(|_| Walk::outside_tests(&found.attrs) == Walk::Entered));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        let found = SignatureReading::free(&self.origins, self.ground).transition(
            self.site(&function.sig.ident),
            &function.sig,
            &function.block,
        );
        self.transitions.extend((Walk::outside_tests(&function.attrs) == Walk::Entered).then_some(found));
    }
}
