use std::iter;

use proc_macro2::TokenTree;
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};
use syn::{Macro, Token};

#[derive(Debug)]
pub(crate) struct MacroCalls(Vec<Macro>);

impl MacroCalls {
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
        steps.collect::<syn::Result<Vec<Vec<Macro>>>>().map(|steps| MacroCalls(steps.into_iter().flatten().collect()))
    }
}

impl From<MacroCalls> for Vec<Macro> {
    fn from(calls: MacroCalls) -> Vec<Macro> {
        calls.0
    }
}
