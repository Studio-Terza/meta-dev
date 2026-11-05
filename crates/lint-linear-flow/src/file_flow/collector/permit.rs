use syn::{Block, Ident, ItemImpl, Signature, TypePath};

use lint_core::Syntax;
use lint_layout::Name;
use lint_placement::CrateName;
use type_facts::Facts;

use super::zone::Zone;
use crate::function_flow::{Root, Target};

#[derive(Clone, Copy, Debug)]
pub enum Permit<'ast> {
    BoolConversion(Target<'ast>),
    Plain,
}

impl<'ast> Permit<'ast> {
    const FROM: &'static str = "From";
    const BOOL: &'static str = "bool";

    pub fn within(zone: Zone, item: &'ast ItemImpl, facts: &'ast Facts<'_>, krate: &CrateName) -> Permit<'ast> {
        match zone {
            Zone::Boundary => Self::bool_source(item)
                .and_then(|_| Self::target(item, facts, krate))
                .map_or(Permit::Plain, Permit::BoolConversion),
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
            .and_then(Syntax::typed)
            .and_then(|typed| Syntax::pat_ident(&typed.pat))
            .map(|binding| &binding.ident)
    }

    fn target(item: &'ast ItemImpl, facts: &'ast Facts<'_>, krate: &CrateName) -> Option<Target<'ast>> {
        Syntax::type_path(&item.self_ty)
            .and_then(|target| target.path.segments.last())
            .map(|segment| &segment.ident)
            .and_then(|name| facts.enum_named(krate, &Name::from(name)).map(|declared| Target::new(name, declared)))
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
