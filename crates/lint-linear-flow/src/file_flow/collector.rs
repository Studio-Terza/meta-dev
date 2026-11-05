mod declaring;
mod permit;
mod zone;

use std::mem;
use std::path::Path;

use syn::visit::{self, Visit};
use syn::{Block, ImplItem, ImplItemFn, ItemFn, ItemImpl, ItemMod, Macro, Signature, TraitItemFn};

use lint_core::{Macros, SourceFile};

use crate::function_flow::FunctionFlow;
use declaring::Declaring;
use permit::Permit;
use zone::Zone;

pub struct Collector<'file> {
    path: &'file Path,
    zone: Zone,
    functions: Vec<FunctionFlow<'file>>,
}

impl<'file> Collector<'file> {
    pub fn collected(file: &'file SourceFile) -> Vec<FunctionFlow<'file>> {
        let mut collector = Collector { path: file.path(), zone: Zone::of_file(file), functions: Vec::new() };
        collector.visit_file(file.syntax());
        collector.functions
    }

    fn visit_member(&mut self, member: &ImplItem, permit: Permit<'_>) {
        match member {
            ImplItem::Fn(function) => self.visit_permitted(function, permit),
            other => self.visit_impl_item(other),
        }
    }

    fn visit_permitted(&mut self, function: &ImplItemFn, permit: Permit<'_>) {
        self.judge(&function.sig, Some(&function.block), permit);
        visit::visit_impl_item_fn(self, function);
    }

    fn judge(&mut self, signature: &Signature, body: Option<&Block>, permit: Permit<'_>) {
        let path = self.path;
        self.functions.extend(
            body.filter(|_| signature.constness.is_none())
                .map(|body| FunctionFlow::judged(path, signature.ident.span(), permit.root(signature, body))),
        );
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Declaring::within(invocation)
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
        let permit = Permit::within(self.zone, item);
        self.visit_generics(&item.generics);
        self.visit_type(&item.self_ty);
        item.items.iter().for_each(|member| self.visit_member(member, permit));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.judge(&function.sig, Some(&function.block), Permit::Plain);
        visit::visit_item_fn(self, function);
    }

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        self.judge(&function.sig, function.default.as_ref(), Permit::Plain);
        visit::visit_trait_item_fn(self, function);
    }
}
