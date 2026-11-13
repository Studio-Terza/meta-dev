use proc_macro2::{TokenStream, TokenTree};
use syn::visit::{self, Visit};
use syn::{Block, ImplItemFn, ItemFn, Macro, Signature, TraitItemFn};

use lint_core::{Macros, SourceFile};

use crate::function_flow::FunctionFlow;

pub struct Collector<'file> {
    file: &'file SourceFile,
    functions: Vec<FunctionFlow<'file>>,
}

impl<'file> Collector<'file> {
    const FUNCTION: &'static str = "fn";

    pub fn collected(file: &'file SourceFile) -> Vec<FunctionFlow<'file>> {
        let mut collector = Collector { file, functions: Vec::new() };
        collector.visit_file(file.syntax());
        collector.functions
    }

    fn declaring(invocation: &Macro) -> Option<&Macro> {
        Self::declares(invocation.tokens.clone()).map(|()| invocation)
    }

    fn declares(tokens: TokenStream) -> Option<()> {
        tokens.into_iter().find_map(Self::declared_in)
    }

    fn declared_in(tree: TokenTree) -> Option<()> {
        match tree {
            TokenTree::Group(group) => Self::declares(group.stream()),
            TokenTree::Ident(ident) => Some(()).filter(|()| ident == Self::FUNCTION),
            TokenTree::Punct(_) | TokenTree::Literal(_) => None,
        }
    }

    fn judge(&mut self, signature: &Signature, body: Option<&Block>) {
        let file = self.file;
        self.functions.extend(
            body.filter(|_| signature.constness.is_none())
                .map(|block| FunctionFlow::judged(file, signature.ident.span(), block)),
        );
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let file = self.file;
        Self::declaring(invocation)
            .into_iter()
            .flat_map(|declared| Macros::expanded(file, declared).to_vec())
            .for_each(|statement| self.visit_stmt(&statement));
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
