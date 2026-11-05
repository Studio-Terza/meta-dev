mod permit;
mod zone;

use std::mem;
use std::path::Path;

use proc_macro2::{TokenStream, TokenTree};
use syn::parse::Parse;
use syn::visit::{self, Visit};
use syn::{Block, ImplItemFn, ItemFn, ItemImpl, ItemMod, Macro, Signature, TraitItemFn};

use lint_core::{Macros, SourceFile};

use crate::function_flow::FunctionFlow;
use permit::Permit;
use zone::Zone;

pub struct Collector<'file> {
    path: &'file Path,
    zone: Zone,
    permit: Permit,
    functions: Vec<FunctionFlow<'file>>,
}

impl<'file> Collector<'file> {
    const FUNCTION: &'static str = "fn";

    pub fn collected(file: &'file SourceFile) -> Vec<FunctionFlow<'file>> {
        let mut collector =
            Collector { path: file.path(), zone: Zone::of_file(file), permit: Permit::Plain, functions: Vec::new() };
        collector.visit_file(file.syntax());
        collector.functions
    }

    fn declaring(invocation: &Macro) -> Option<&Macro> {
        invocation
            .parse_body_with(TokenStream::parse)
            .into_iter()
            .flat_map(TokenStream::into_iter)
            .flat_map(Self::flattened)
            .find(|tree| matches!(tree, TokenTree::Ident(ident) if ident == Self::FUNCTION))
            .map(|_| invocation)
    }

    fn flattened(tree: TokenTree) -> Vec<TokenTree> {
        match tree {
            TokenTree::Group(group) => group.stream().into_iter().flat_map(Self::flattened).collect(),
            other => vec![other],
        }
    }

    fn judge(&mut self, signature: &Signature, body: Option<&Block>) {
        let path = self.path;
        let permit = self.permit;
        self.functions.extend(
            body.filter(|_| signature.constness.is_none())
                .map(|body| FunctionFlow::judged(path, signature.ident.span(), permit.root(body))),
        );
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Self::declaring(invocation)
            .into_iter()
            .flat_map(Macros::statements)
            .for_each(|statement| self.visit_stmt(&statement));
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let entered = self.zone.entering(module);
        let outer = mem::replace(&mut self.zone, entered);
        visit::visit_item_mod(self, module);
        self.zone = outer;
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let permitted = Permit::within(self.zone, item);
        let outer = mem::replace(&mut self.permit, permitted);
        visit::visit_item_impl(self, item);
        self.permit = outer;
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
