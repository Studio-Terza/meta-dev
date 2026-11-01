mod spelled;

use std::iter;

use proc_macro2::{Ident, TokenStream, TokenTree};
use syn::Path;
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};

use lint_layout::{LineNumber, UsePath};
pub(crate) use spelled::Spelled;

#[derive(Debug)]
pub(crate) struct TokenPaths(Vec<Spelled>);

impl TokenPaths {
    const ACCESS: char = '.';

    pub(crate) fn within(tokens: TokenStream) -> Vec<Spelled> {
        syn::parse2::<TokenPaths>(tokens).map(|paths| paths.0).unwrap_or_default()
    }

    fn step(input: ParseStream) -> syn::Result<Vec<Spelled>> {
        let attempt = input.fork();
        match attempt.call(Path::parse_mod_style) {
            Ok(path) => {
                input.advance_to(&attempt);
                let first = path.segments.first().map(|first| LineNumber::from(first.ident.span()));
                Ok(first.map(|line| Spelled::new(UsePath::from(&path), line)).into_iter().collect())
            }
            Err(_) => Self::skipped(input.parse()?, input),
        }
    }

    fn skipped(tree: TokenTree, input: ParseStream) -> syn::Result<Vec<Spelled>> {
        match tree {
            TokenTree::Group(group) => Ok(Self::within(group.stream())),
            TokenTree::Punct(punct) if punct.as_char() == Self::ACCESS => {
                input.parse::<Option<Ident>>().map(|_| Vec::new())
            }
            _ => Ok(Vec::new()),
        }
    }
}

impl Parse for TokenPaths {
    fn parse(input: ParseStream) -> syn::Result<TokenPaths> {
        let steps = iter::from_fn(|| (!input.is_empty()).then(|| Self::step(input)));
        steps.collect::<syn::Result<Vec<Vec<Spelled>>>>().map(|steps| TokenPaths(steps.into_iter().flatten().collect()))
    }
}
