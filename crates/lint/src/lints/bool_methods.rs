use syn::visit::{self, Visit};
use syn::{Attribute, ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemTrait, ReturnType, Signature, TraitItemFn, Type};

use crate::finding::{Found, Rule};
use crate::gate::Gate;
use crate::source::Source;
use crate::style::{Reads, Style};

#[derive(Debug, Default)]
pub(crate) struct BoolMethods {
    found: Vec<Found>,
}

impl BoolMethods {
    const FIX: &'static str =
        "return an enum to match on, an Option or Result carrying the value, or a proof type such as NonEmpty";

    fn opened(&mut self, attributes: &[Attribute], signature: &Signature) -> Option<()> {
        Gate::of(attributes).entered()?;
        let flag = match &signature.output {
            ReturnType::Type(_, returned) => match &**returned {
                Type::Path(path) => (path.qself.is_none() && path.path.is_ident("bool")).then_some(()),
                _ => None,
            },
            ReturnType::Default => None,
        };
        let name = &signature.ident;
        self.found
            .extend(flag.map(|()| Found::spanned(name.span(), format_args!("fn {name} -> bool"), BoolMethods::FIX)));
        Some(())
    }
}

impl<'ast> Visit<'ast> for BoolMethods {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Gate::of(&module.attrs).entered().map_or_default(|()| visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let walked = Gate::inherent(item).and(Gate::of(&item.attrs));
        walked.entered().map_or_default(|()| visit::visit_item_impl(self, item));
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        Gate::of(&item.attrs).entered().map_or_default(|()| visit::visit_item_trait(self, item));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.opened(&function.attrs, &function.sig).map_or_default(|()| visit::visit_item_fn(self, function));
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.opened(&function.attrs, &function.sig).map_or_default(|()| visit::visit_impl_item_fn(self, function));
    }

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        self.opened(&function.attrs, &function.sig).map_or_default(|()| visit::visit_trait_item_fn(self, function));
    }
}

impl<'file> Style<'file> for BoolMethods {
    const RULE: Rule = Rule::BoolMethods;
    const READS: Reads = Reads::Shipped;

    fn new(_: &'file Source) -> BoolMethods {
        BoolMethods::default()
    }

    fn found(self) -> Vec<Found> {
        self.found
    }
}
