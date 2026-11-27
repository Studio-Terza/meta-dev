use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{FnArg, GenericArgument, ItemImpl, ItemMod, Macro, PathArguments, Signature, Type};

use crate::expansion::Expansion;
use crate::finding::{Fix, Found, Rule};
use crate::gate::Gate;
use crate::source::Source;
use crate::style::{Reads, Style};
use crate::word::Word;

#[derive(Debug, Default)]
pub(crate) struct VecOutParams {
    found: Vec<Found>,
}

impl VecOutParams {
    const FIX: Fix = Fix("give the collection a type that owns its rules");
    const VEC: Word = Word("Vec");
    const OPTION: Word = Word("Option");

    fn vec_out(parameter: &Type) -> Option<()> {
        match parameter {
            Type::Reference(reference) => {
                reference.mutability.and_then(|_| VecOutParams::VEC.last(&reference.elem)).map(|_| ())
            }
            Type::Path(_) => VecOutParams::OPTION
                .last(parameter)
                .and_then(|option| match &option.arguments {
                    PathArguments::AngleBracketed(arguments) => arguments.args.first(),
                    PathArguments::None | PathArguments::Parenthesized(_) => None,
                })
                .and_then(|argument| match argument {
                    GenericArgument::Type(inner) => VecOutParams::vec_out(inner),
                    _ => None,
                }),
            _ => None,
        }
    }
}

impl<'ast> Visit<'ast> for VecOutParams {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Gate::of(&module.attrs).entered().map_or_default(|()| visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Gate::inherent(item).entered().map_or_default(|()| visit::visit_item_impl(self, item));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Expansion::of(invocation).walk(self);
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        let name = &signature.ident;
        let parameters = signature.inputs.iter().filter_map(|input| match input {
            FnArg::Typed(typed) => Some(typed),
            FnArg::Receiver(_) => None,
        });
        let out = parameters.filter(|parameter| VecOutParams::vec_out(&parameter.ty).is_some());
        let found: Vec<Found> = out
            .map(|parameter| Found::spanned(parameter.pat.span(), format_args!("fn {name}"), VecOutParams::FIX))
            .collect();
        self.found.extend(found);
    }
}

impl<'file> Style<'file> for VecOutParams {
    const RULE: Rule = Rule::VecOutParams;
    const READS: Reads = Reads::Every;

    fn new(_: &'file Source) -> VecOutParams {
        VecOutParams::default()
    }

    fn found(self) -> Vec<Found> {
        self.found
    }
}
