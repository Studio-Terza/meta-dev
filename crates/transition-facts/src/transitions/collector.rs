mod signature_reading;

use syn::visit::{self, Visit};
use syn::{Ident, ImplItem, ItemFn, ItemImpl, ItemMod, ItemTrait, TraitItem};

use lint_core::{Site, SourceFile, Walk};
use lint_layout::LineNumber;
use lint_placement::CrateName;

use crate::ground::Ground;
use crate::transition::Transition;
use signature_reading::SignatureReading;

pub(super) struct Collector<'file> {
    file: &'file SourceFile,
    krate: CrateName,
    ground: Ground,
    transitions: Vec<Transition<'file>>,
}

impl<'file> Collector<'file> {
    pub(super) fn collected(file: &'file SourceFile) -> Vec<Transition<'file>> {
        let mut scan = Collector {
            file,
            krate: CrateName::from(file.placement().package().root()),
            ground: Ground::gated(file),
            transitions: Vec::new(),
        };
        scan.visit_file(file.syntax());
        scan.transitions
    }

    fn site(file: &'file SourceFile, name: &Ident) -> Site<'file> {
        Site::new(file.path(), LineNumber::from(name.span()))
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
        let reading = SignatureReading::of_trait(self.krate.clone(), found, self.ground);
        let file = self.file;
        let provided = found
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
                    .map(|body| reading.transition(Self::site(file, &function.sig.ident), &function.sig, body))
            });
        self.transitions.extend(provided.filter(|_| Walk::outside_tests(&found.attrs) == Walk::Entered));
    }

    fn visit_item_impl(&mut self, found: &'ast ItemImpl) {
        let reading = SignatureReading::of_impl(self.krate.clone(), found, self.ground);
        let file = self.file;
        let transitions = found.items.iter().filter_map(|item| match item {
            ImplItem::Fn(function) if Walk::outside_tests(&function.attrs) == Walk::Entered => {
                Some(reading.transition(Self::site(file, &function.sig.ident), &function.sig, &function.block))
            }
            _ => None,
        });
        self.transitions.extend(transitions.filter(|_| Walk::outside_tests(&found.attrs) == Walk::Entered));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        let found = SignatureReading::free(self.krate.clone(), self.ground).transition(
            Self::site(self.file, &function.sig.ident),
            &function.sig,
            &function.block,
        );
        self.transitions.extend((Walk::outside_tests(&function.attrs) == Walk::Entered).then_some(found));
    }
}
