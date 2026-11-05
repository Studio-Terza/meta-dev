use syn::{Block, FnArg, Ident, ItemImpl, PatType, Signature, TypePath};

use lint_core::Syntax;

use super::zone::Zone;
use crate::function_flow::Root;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Permit<'ast> {
    BoolConversion(&'ast Ident),
    Plain,
}

impl<'ast> Permit<'ast> {
    const FROM: &'static str = "From";
    const BOOL: &'static str = "bool";

    pub fn within(zone: Zone, item: &'ast ItemImpl) -> Permit<'ast> {
        match zone {
            Zone::Boundary => {
                Self::bool_source(item).and_then(|_| Self::target(item)).map_or(Permit::Plain, Permit::BoolConversion)
            }
            Zone::Elsewhere => Permit::Plain,
        }
    }

    pub fn root(self, signature: &'ast Signature, body: &'ast Block) -> Root<'ast> {
        match self {
            Permit::BoolConversion(target) => Self::sole_input(signature)
                .map_or_else(|| Root::from(body), |input| Root::converting(body, input, target)),
            Permit::Plain => Root::from(body),
        }
    }

    fn sole_input(signature: &Signature) -> Option<&Ident> {
        signature
            .inputs
            .first()
            .filter(|_| signature.inputs.len() == 1)
            .and_then(Self::typed)
            .and_then(|typed| Syntax::pat_ident(&typed.pat))
            .map(|binding| &binding.ident)
    }

    fn typed(argument: &FnArg) -> Option<&PatType> {
        match argument {
            FnArg::Typed(typed) => Some(typed),
            FnArg::Receiver(_) => None,
        }
    }

    fn target(item: &ItemImpl) -> Option<&Ident> {
        Syntax::type_path(&item.self_ty).and_then(|target| target.path.segments.last()).map(|segment| &segment.ident)
    }

    fn bool_source(item: &ItemImpl) -> Option<&TypePath> {
        item.trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .filter(|segment| segment.ident == Self::FROM)
            .and_then(|segment| Syntax::angled(&segment.arguments))
            .and_then(|angled| angled.args.first())
            .and_then(Syntax::type_argument)
            .and_then(Syntax::type_path)
            .filter(|source| source.path.is_ident(Self::BOOL))
    }
}
