use syn::spanned::Spanned;
use syn::{FnArg, GenericArgument, ItemImpl, ItemMod, Macro, PathArguments, Signature, Type, TypePath};

use crate::finding::{Found, Rule};
use crate::gate::Gate;
use crate::hooks::{Descent, Expanded, Hooks};
use crate::source::Source;
use crate::style::{Reads, Style};
use crate::terms::{Fix, Word};

#[derive(Debug, Default)]
pub(crate) struct VecOutParams {
    found: Vec<Found>,
}

impl VecOutParams {
    const FIX: Fix = Fix("give the collection a type that owns its rules");

    fn vec_out(parameter: &Type) -> Option<()> {
        match parameter {
            Type::Reference(reference) => {
                reference.mutability.and_then(|_| VecOutParams::last_named(&reference.elem, Word("Vec"))).map(|_| ())
            }
            Type::Path(_) => VecOutParams::last_named(parameter, Word("Option"))
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

    fn last_named(ty: &Type, name: Word) -> Option<&syn::PathSegment> {
        match ty {
            Type::Path(TypePath { path, .. }) => path.segments.last().filter(|segment| segment.ident == name),
            _ => None,
        }
    }
}

impl Hooks for VecOutParams {
    fn item_mod(&mut self, module: &ItemMod) -> Descent {
        Descent::from(Gate::of(&module.attrs).entered())
    }

    fn item_impl(&mut self, item: &ItemImpl) -> Descent {
        Descent::from(Gate::inherent(item).entered())
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Statements
    }

    fn signature(&mut self, signature: &Signature) -> Descent {
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
        Descent::Over
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
