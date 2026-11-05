use std::iter;

use lint_core::MacroAliases;
use lint_layout::{Name, Whole, Word};

use super::body::Body;
use super::scope::Scope;
use crate::check::{Check, Domain};
use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Block, Ident, Macro, ReturnType, Token, Visibility, parenthesized};

pub(crate) struct Property {
    site: Ident,
    body: Block,
}

impl Property {
    pub(crate) fn site(&self) -> &Ident {
        &self.site
    }

    pub(crate) fn checks(&self, scope: &Scope<'_>, tables: &MacroAliases) -> Vec<Check> {
        Body::checks(scope, tables, &self.body, Domain::GENERATED)
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
        let inputs;
        parenthesized!(inputs in input);
        inputs.parse::<TokenStream>()?;
        input.parse::<ReturnType>()?;
        Ok(Property { site, body: input.parse()? })
    }
}
