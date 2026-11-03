use std::mem;

use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Attribute, Block, ExprClosure, ExprMatch, ImplItemFn, ItemFn, ItemImpl, ItemMod, TraitItemFn};

use body_shape::Shape;
use lint_core::{Bound, Site, SourceFile, Walk};
use lint_layout::{LineNumber, TestMarks};

use crate::extent::Extent;
use crate::kind::Kind;
use crate::node::Node;
use crate::occurrence::Occurrence;

pub(crate) struct Scan<'file> {
    file: &'file SourceFile,
    kind: Kind,
    nodes: Vec<Node<'file>>,
}

impl<'file> Scan<'file> {
    const FLOOR: Bound = Bound::Least(40);

    pub(crate) fn of_file(file: &'file SourceFile) -> Vec<Node<'file>> {
        let tests = file.test_modules().iter().flat_map(|module| Self::scanned(module, Kind::Tests));
        Self::scanned(file, Kind::of_role(file.placement().role())).into_iter().chain(tests).collect()
    }

    fn scanned(file: &'file SourceFile, kind: Kind) -> Vec<Node<'file>> {
        let mut scan = Scan { file, kind, nodes: Vec::new() };
        scan.visit_file(file.syntax());
        scan.nodes
    }

    fn considered(&mut self, node: &(impl ToTokens + Spanned)) {
        let shape = Shape::from(node.to_token_stream());
        let span = node.span();
        let site = Site::new(self.file.path(), LineNumber::from(span));
        let occurrence = Occurrence::new(site, Extent::from(span), self.kind);
        self.nodes.extend(Self::FLOOR.allows(shape.size()).map(|_| Node::new(shape, occurrence)));
    }

    fn within(&mut self, attributes: &[Attribute], visit: impl FnOnce(&mut Self)) {
        let inner = match (self.kind, Walk::outside_tests(attributes), TestMarks::mark_test(attributes)) {
            (Kind::Shipped, Walk::Entered, None) => Kind::Shipped,
            _ => Kind::Tests,
        };
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
