use std::path::Path;

use syn::visit::{self, Visit};
use syn::{Block, ImplItemFn, ItemFn, Macro, Signature, TraitItemFn};

use lint_core::Macros;

use crate::function_flow::FunctionFlow;

pub struct Collector<'file> {
    path: &'file Path,
    functions: Vec<FunctionFlow<'file>>,
}

impl<'file> Collector<'file> {
    pub fn collected(path: &'file Path, syntax: &syn::File) -> Vec<FunctionFlow<'file>> {
        let mut collector = Collector { path, functions: Vec::new() };
        collector.visit_file(syntax);
        collector.functions
    }

    fn judge(&mut self, signature: &Signature, body: Option<&Block>) {
        let path = self.path;
        self.functions.extend(
            body.filter(|_| signature.constness.is_none())
                .map(|body| FunctionFlow::judged(path, signature.ident.span(), body)),
        );
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.judge(&function.sig, Some(&function.block));
        visit::visit_item_fn(self, function);
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.judge(&function.sig, Some(&function.block));
        visit::visit_impl_item_fn(self, function);
    }

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        self.judge(&function.sig, function.default.as_ref());
        visit::visit_trait_item_fn(self, function);
    }
}
