use std::iter;

use proc_macro2::{TokenStream, TokenTree};
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};
use syn::{Macro, Token};

#[derive(Debug)]
pub struct MacroCalls(Vec<Macro>);

impl MacroCalls {
    const BANG: char = '!';

    #[must_use]
    pub fn within(invocation: &Macro) -> Vec<Macro> {
        Self::bangs(&invocation.tokens)
            .map_or_default(|()| syn::parse2::<MacroCalls>(invocation.tokens.clone()).map_or_default(|calls| calls.0))
    }

    fn bangs(tokens: &TokenStream) -> Option<()> {
        tokens.clone().into_iter().find_map(Self::bang)
    }

    fn bang(tree: TokenTree) -> Option<()> {
        match tree {
            TokenTree::Punct(punct) => Some(punct.as_char()).filter(|spelled| *spelled == Self::BANG).map(|_| ()),
            TokenTree::Group(group) => Self::bangs(&group.stream()),
            TokenTree::Ident(_) | TokenTree::Literal(_) => None,
        }
    }

    fn step(input: ParseStream) -> syn::Result<Vec<Macro>> {
        let metavariable = input.parse::<Option<Token![$]>>()?;
        let attempt = input.fork();
        attempt.parse::<Macro>().map_or_else(
            |_| input.parse().and_then(Self::skipped),
            |found| {
                input.advance_to(&attempt);
                Ok(metavariable.map_or_else(|| vec![found], |_| Vec::new()))
            },
        )
    }

    fn skipped(tree: TokenTree) -> syn::Result<Vec<Macro>> {
        match tree {
            TokenTree::Group(group) => syn::parse2::<MacroCalls>(group.stream()).map(|calls| calls.0),
            _ => Ok(Vec::new()),
        }
    }
}

impl Parse for MacroCalls {
    fn parse(input: ParseStream) -> syn::Result<MacroCalls> {
        let steps = iter::from_fn(|| input.cursor().token_tree().map(|_| Self::step(input)));
        steps.collect::<syn::Result<Vec<Vec<Macro>>>>().map(|parsed| MacroCalls(parsed.into_iter().flatten().collect()))
    }
}
