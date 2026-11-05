use std::mem;

use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Attribute, Block, ExprClosure, ExprMatch, ImplItemFn, ItemFn, ItemImpl, ItemMod, TraitItemFn};

use body_shape::Reading;
use equivalence::Equivalence;
use lint_core::{Bound, Site, SourceFile, Walk};
use lint_layout::{LineNumber, TestMarks};

use crate::extent::Extent;
use crate::kind::Kind;
use crate::occurrence::Occurrence;

pub(crate) struct Scan<'file> {
    file: &'file SourceFile,
    kind: Kind,
    equivalence: Equivalence,
    occurrences: Vec<Occurrence<'file>>,
}

impl<'file> Scan<'file> {
    const FLOOR: Bound = Bound::Least(40);

    pub(crate) fn of_file(file: &'file SourceFile, equivalence: Equivalence) -> Vec<Occurrence<'file>> {
        let tests = file.test_modules().iter().flat_map(|module| Self::scanned(module, Kind::Tests, equivalence));
        Self::scanned(file, Kind::of_role(file.placement().role()), equivalence).into_iter().chain(tests).collect()
    }

    fn scanned(file: &'file SourceFile, kind: Kind, equivalence: Equivalence) -> Vec<Occurrence<'file>> {
        let mut scan = Scan { file, kind, equivalence, occurrences: Vec::new() };
        scan.visit_file(file.syntax());
        scan.occurrences
    }

    fn considered(&mut self, node: &(impl ToTokens + Spanned)) {
        let reading = Reading::erasing(node.to_token_stream(), self.equivalence);
        let span = node.span();
        let site = Site::new(self.file.path(), LineNumber::from(span));
        let tokens = reading.shape().size();
        let occurrence = Occurrence::new(reading, site, Extent::from(span), self.kind);
        self.occurrences.extend(Self::FLOOR.allows(tokens).map(|_| occurrence));
    }

    fn within(&mut self, attributes: &[Attribute], visit: impl FnOnce(&mut Self)) {
        let inner = Some(self.kind)
            .filter(|kind| *kind == Kind::Shipped)
            .filter(|_| Walk::outside_tests(attributes) == Walk::Entered)
            .filter(|_| TestMarks::mark_test(attributes).is_none())
            .unwrap_or(Kind::Tests);
        let outer = mem::replace(&mut self.kind, inner);
        visit(self);
        self.kind = outer;
    }
}

impl<'ast> Visit<'ast> for Scan<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.within(&module.attrs, |scan| visit::visit_item_mod(scan, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.within(&item.attrs, |scan| visit::visit_item_impl(scan, item));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.within(&function.attrs, |scan| visit::visit_item_fn(scan, function));
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.within(&function.attrs, |scan| visit::visit_impl_item_fn(scan, function));
    }

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        self.within(&function.attrs, |scan| visit::visit_trait_item_fn(scan, function));
    }

    fn visit_block(&mut self, block: &'ast Block) {
        self.considered(block);
        visit::visit_block(self, block);
    }

    fn visit_expr_match(&mut self, found: &'ast ExprMatch) {
        self.considered(found);
        visit::visit_expr_match(self, found);
    }

    fn visit_expr_closure(&mut self, found: &'ast ExprClosure) {
        self.considered(found);
        visit::visit_expr_closure(self, found);
    }
}
