use proc_macro2::{Group, TokenStream, TokenTree};
use quote::quote;
use syn::parse::discouraged::Speculative;
use syn::parse::{ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{Attribute, Block, Expr, Macro, Stmt, Token};

use crate::syntax::Syntax;

#[derive(Debug)]
pub struct Macros;

impl Macros {
    const ARROW_END: char = '>';

    #[must_use]
    pub fn statements(invocation: &Macro) -> Vec<Stmt> {
        let bodies = invocation
            .path
            .get_ident()
            .filter(|ident| *ident == "macro_rules")
            .map_or_else(|| vec![invocation.tokens.clone()], |_| Self::transcribers(invocation.tokens.clone()));
        bodies
            .into_iter()
            .flat_map(|tokens| Self::statements_or_expressions.parse2(tokens).unwrap_or_default())
            .collect()
    }

    #[must_use]
    pub fn arguments(invocation: &Macro) -> Vec<Expr> {
        invocation
            .parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            .map(|arguments| arguments.into_iter().collect())
            .unwrap_or_default()
    }

    pub fn walk(invocation: &Macro, visit: impl FnMut(&Stmt)) {
        Self::statements(invocation).iter().for_each(visit);
    }

    #[must_use]
    pub fn attributes(invocation: &Macro) -> Vec<Attribute> {
        Self::attributes_in(invocation.tokens.clone())
    }

    fn attributes_in(tokens: TokenStream) -> Vec<Attribute> {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        let here = trees.iter().zip(trees.iter().skip(1)).filter_map(|(mark, body)| {
            Syntax::punct(mark)
                .and(Syntax::group(body))
                .and_then(|_| Attribute::parse_outer.parse2(quote!(#mark #body)).ok())
        });
        let nested = trees.iter().filter_map(Syntax::group).map(Group::stream);
        here.flatten().chain(nested.flat_map(Self::attributes_in)).collect()
    }

    fn statements_or_expressions(input: ParseStream) -> syn::Result<Vec<Stmt>> {
        let attempt = input.fork();
        Block::parse_within(&attempt).map_or_else(
            |_| Punctuated::<Expr, Token![,]>::parse_terminated(input).map(Self::expressions),
            |statements| {
                input.advance_to(&attempt);
                Ok(statements)
            },
        )
    }

    fn expressions(arguments: Punctuated<Expr, Token![,]>) -> Vec<Stmt> {
        arguments.into_iter().map(|argument| Stmt::Expr(argument, None)).collect()
    }

    fn transcribers(rules: TokenStream) -> Vec<TokenStream> {
        let trees: Vec<TokenTree> = rules.into_iter().collect();
        trees
            .iter()
            .zip(trees.iter().skip(1))
            .filter_map(|(arrow, body)| {
                Syntax::punct(arrow).filter(|punct| punct.as_char() == Self::ARROW_END).and(Syntax::group(body))
            })
            .map(Group::stream)
            .collect()
    }
}
