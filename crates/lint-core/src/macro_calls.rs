use std::iter;

use proc_macro2::TokenTree;
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};
use syn::{Macro, Token};

#[derive(Debug)]
pub struct MacroCalls(Vec<Macro>);

impl MacroCalls {
    #[must_use]
    pub fn within(invocation: &Macro) -> Vec<Macro> {
        syn::parse2::<MacroCalls>(invocation.tokens.clone()).map(|calls| calls.0).unwrap_or_default()
    }

    fn step(input: ParseStream) -> syn::Result<Vec<Macro>> {
        let metavariable = input.parse::<Option<Token![$]>>()?;
        let attempt = input.fork();
        match attempt.parse::<Macro>() {
            Ok(found) => {
                input.advance_to(&attempt);
                Ok(vec![found].into_iter().filter(|_| metavariable.is_none()).collect())
            }
            Err(_) => Self::skipped(input.parse()?),
        }
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
        let steps = iter::from_fn(|| (!input.is_empty()).then(|| Self::step(input)));
        steps.collect::<syn::Result<Vec<Vec<Macro>>>>().map(|steps| MacroCalls(steps.into_iter().flatten().collect()))
    }
}
