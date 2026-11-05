use std::iter;

use proc_macro2::{Delimiter, Group, Ident, TokenStream, TokenTree};
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::{Macro, Token};

use lint_layout::Name;

#[derive(Debug)]
pub(crate) struct TokenCalls(Vec<Name>);

#[derive(Debug)]
struct Called(Name);

impl Called {
    const NOT_A_CALL: &'static str = "not a method call";
}

impl Parse for Called {
    fn parse(attempt: ParseStream) -> syn::Result<Called> {
        attempt.parse::<Token![.]>()?;
        let method: Ident = attempt.parse()?;
        attempt
            .fork()
            .parse::<Group>()
            .ok()
            .filter(|arguments| arguments.delimiter() == Delimiter::Parenthesis)
            .map(|_| Called(Name::from(&method)))
            .ok_or_else(|| attempt.error(Self::NOT_A_CALL))
    }
}

impl TokenCalls {
    pub(crate) fn called_in(invocation: &Macro) -> Vec<Name> {
        invocation.parse_body::<TokenCalls>().map(|calls| calls.0).unwrap_or_default()
    }

    fn within(tokens: TokenStream) -> Vec<Name> {
        syn::parse2::<TokenCalls>(tokens).map(|calls| calls.0).unwrap_or_default()
    }

    fn attempted(input: ParseStream, attempt: &ParseBuffer) -> syn::Result<Vec<Name>> {
        match attempt.parse::<Called>() {
            Ok(Called(method)) => Ok(Self::advanced(input, attempt, method)),
            Err(_) => Ok(Self::skipped(input.parse()?)),
        }
    }

    fn advanced(input: ParseStream, attempt: &ParseBuffer, method: Name) -> Vec<Name> {
        input.advance_to(attempt);
        vec![method]
    }

    fn skipped(tree: TokenTree) -> Vec<Name> {
        match tree {
            TokenTree::Group(group) => Self::within(group.stream()),
            TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => Vec::new(),
        }
    }
}

impl Parse for TokenCalls {
    fn parse(input: ParseStream) -> syn::Result<TokenCalls> {
        let steps = iter::from_fn(|| input.cursor().token_tree().map(|_| Self::attempted(input, &input.fork())));
        steps.collect::<syn::Result<Vec<Vec<Name>>>>().map(|steps| TokenCalls(steps.into_iter().flatten().collect()))
    }
}
