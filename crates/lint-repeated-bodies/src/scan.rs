use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Block, ExprClosure, ExprMatch, ExprMethodCall, ImplItemFn, ItemFn, ItemMod, Signature, TraitItemFn,
};

use body_shape::Shape;
use lint_core::{Least, Site, SourceFile, Walk};
use lint_layout::LineNumber;
use newtype::Counter;

use crate::extent::Extent;
use crate::node::Node;
use crate::occurrence::Occurrence;

pub(crate) struct Scan<'file> {
    file: &'file SourceFile,
    nodes: Vec<Node>,
}

impl Scan<'_> {
    const FLOOR: Least = Least::new(40);

    pub(crate) fn of_file(file: &SourceFile) -> Vec<Node> {
        let mut scan = Scan { file, nodes: Vec::new() };
        scan.visit_file(file.syntax());
        scan.nodes
    }

    fn considered(&mut self, node: &(impl ToTokens + Spanned)) {
        let shape = Shape::from(node.to_token_stream());
        let span = node.span();
        let occurrence = Occurrence::new(Site::new(self.file.path(), LineNumber::from(span)), Extent::from(span));
        self.nodes.extend(Self::FLOOR.allows(shape.size().count()).then(|| Node::new(shape, occurrence)));
    }

    fn gate(attributes: &[Attribute], signature: &Signature) -> Walk {
        match (signature.constness, Walk::outside_tests(attributes)) {
            (None, Walk::Entered) => Walk::Entered,
            _ => Walk::Skipped,
        }
    }

    fn entered(walk: Walk, visit: impl FnOnce()) {
        match walk {
            Walk::Entered => visit(),
            Walk::Skipped => {}
        }
    }
}

impl<'ast> Visit<'ast> for Scan<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        Self::entered(Self::gate(&function.attrs, &function.sig), || visit::visit_item_fn(self, function));
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        Self::entered(Self::gate(&function.attrs, &function.sig), || visit::visit_impl_item_fn(self, function));
    }

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        Self::entered(Self::gate(&function.attrs, &function.sig), || visit::visit_trait_item_fn(self, function));
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

    fn visit_expr_method_call(&mut self, found: &'ast ExprMethodCall) {
        self.considered(found);
        visit::visit_expr_method_call(self, found);
    }
}
