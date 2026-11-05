use std::iter;

use lint_layout::{Name, Whole, Word};

use super::body::Body;
use super::scope::{Place, Scope};
use super::trace::{Extent, Locals, Trace};
use crate::check::{Breadth, Check, Domain, Rows};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Block, Expr, Ident, Macro, Pat, ReturnType, Token, Type, Visibility, parenthesized};

enum Input {
    Strategy(Expr),
    Declared(Type),
}

impl Input {
    fn rows(&self, trace: &Trace<'_, '_>) -> Option<Rows> {
        match self {
            Input::Strategy(strategy) => Extent::strategy(trace, strategy),
            Input::Declared(declared) => Extent::typed(declared),
        }
    }
}

impl Input {
    fn declared(input: ParseStream) -> syn::Result<Input> {
        input.parse::<Token![:]>()?;
        Ok(Input::Declared(input.parse()?))
    }
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Input> {
        input.call(Attribute::parse_outer)?;
        input.call(Pat::parse_single)?;
        let strategy = input.parse::<Option<Token![in]>>()?.map(|_| input.parse::<Expr>()).transpose()?;
        strategy.map_or_else(|| Input::declared(input), |strategy| Ok(Input::Strategy(strategy)))
    }
}

pub(crate) struct Property {
    site: Ident,
    inputs: Vec<Input>,
    body: Block,
}

impl Property {
    pub(crate) fn site(&self) -> &Ident {
        &self.site
    }

    pub(crate) fn checks(&self, scope: &Scope<'_>, place: &Place) -> Vec<Check> {
        let locals = Locals::default();
        let trace = Trace::new(scope, &locals, place);
        let domain = self
            .inputs
            .iter()
            .filter_map(|input| input.rows(&trace))
            .map(|rows| Domain::new(Breadth::Generated, rows))
            .fold(Domain::GENERATED, Domain::max_with);
        Body::checks(scope, place, &self.body, domain)
    }
}

pub(crate) struct Properties {
    found: Vec<Property>,
}

impl Properties {
    const PROPTEST: Word<'static> = Word::new("proptest");

    pub(crate) fn declared(invocation: &Macro) -> Vec<Property> {
        invocation
            .path
            .segments
            .last()
            .and_then(|last| Name::from(&last.ident).matches::<Whole>(&[Self::PROPTEST]))
            .and_then(|_| invocation.parse_body_with(Properties::parse).ok())
            .map_or_else(Vec::new, |properties| properties.found)
    }
}

impl Parse for Properties {
    fn parse(input: ParseStream) -> syn::Result<Properties> {
        input.call(Attribute::parse_inner)?;
        let found = iter::from_fn(|| input.is_empty().then_some(()).map_or_else(|| Some(input.parse()), |()| None))
            .collect::<syn::Result<Vec<Property>>>()?;
        Ok(Properties { found })
    }
}

impl Parse for Property {
    fn parse(input: ParseStream) -> syn::Result<Property> {
        input.call(Attribute::parse_outer)?;
        input.parse::<Visibility>()?;
        input.parse::<Token![fn]>()?;
        let site: Ident = input.parse()?;
        let arguments;
        parenthesized!(arguments in input);
        let inputs = Punctuated::<Input, Token![,]>::parse_terminated(&arguments)?.into_iter().collect();
        input.parse::<ReturnType>()?;
        Ok(Property { site, inputs, body: input.parse()? })
    }
}
