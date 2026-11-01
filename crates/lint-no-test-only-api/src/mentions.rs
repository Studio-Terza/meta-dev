mod ground;

use std::collections::BTreeSet;

use proc_macro2::{TokenStream, TokenTree};
use syn::visit::{self, Visit};
use syn::{Attribute, Ident, ImplItemFn, ItemFn, ItemImpl, ItemMod, Macro, Signature, Variant};

use lint_core::{SourceFile, Walk};
use lint_layout::Name;

pub(crate) use ground::Ground;

#[derive(Debug)]
pub(crate) struct Mentions {
    wanted: Ground,
    ground: Ground,
    names: BTreeSet<Name>,
}

impl Mentions {
    pub(crate) fn in_file(file: &SourceFile, wanted: Ground) -> BTreeSet<Name> {
        let ground = if file.is_shipped() { Ground::Shipped } else { Ground::Tests };
        let mut mentions = Mentions { wanted, ground, names: BTreeSet::new() };
        mentions.visit_file(file.syntax());
        file.test_modules().iter().for_each(|module| {
            mentions.within(Ground::Tests, |inner| inner.visit_file(module.syntax()));
        });
        mentions.names
    }

    fn within(&mut self, ground: Ground, visit: impl FnOnce(&mut Mentions)) {
        let before = self.ground;
        self.ground = ground;
        visit(self);
        self.ground = before;
    }

    fn entered(&mut self, attributes: &[Attribute], visit: impl FnOnce(&mut Mentions)) {
        match Walk::outside_tests(attributes) {
            Walk::Entered => visit(self),
            Walk::Skipped => self.within(Ground::Tests, visit),
        }
    }

    fn named(&mut self, name: Name) {
        self.names.extend((self.ground == self.wanted).then_some(name));
    }

    fn scanned(&mut self, tokens: TokenStream) {
        tokens.into_iter().for_each(|tree| match tree {
            TokenTree::Ident(ident) => self.named(Name::from(&ident)),
            TokenTree::Group(group) => self.scanned(group.stream()),
            TokenTree::Punct(_) | TokenTree::Literal(_) => {}
        });
    }
}

impl<'ast> Visit<'ast> for Mentions {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        self.named(Name::from(ident));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.scanned(invocation.tokens.clone());
        visit::visit_macro(self, invocation);
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.entered(&module.attrs, |inner| visit::visit_item_mod(inner, module));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.entered(&function.attrs, |inner| {
            inner.visit_signature(&function.sig);
            inner.visit_block(&function.block);
        });
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.entered(&item.attrs, |inner| visit::visit_item_impl(inner, item));
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.entered(&function.attrs, |inner| {
            inner.visit_signature(&function.sig);
            inner.visit_block(&function.block);
        });
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        self.visit_generics(&signature.generics);
        signature.inputs.iter().for_each(|input| self.visit_fn_arg(input));
        self.visit_return_type(&signature.output);
    }

    fn visit_variant(&mut self, variant: &'ast Variant) {
        variant.fields.iter().for_each(|field| self.visit_field(field));
        variant.discriminant.iter().for_each(|(_, discriminant)| self.visit_expr(discriminant));
    }
}
