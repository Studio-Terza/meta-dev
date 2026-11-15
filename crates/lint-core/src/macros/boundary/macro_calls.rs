use std::iter;

use syntax::parse::Speculative;
use syntax::parse::{Parse, ParseStream};
use syntax::{Macro, Token};
use syntax::{Punct, TokenStream, TokenTree};

#[derive(Debug)]
pub struct MacroCalls(Vec<Macro>);

impl MacroCalls {
    const BANG: char = '!';

    #[must_use]
    pub fn within(invocation: &Macro) -> Vec<Macro> {
        Self::banged(invocation.tokens.clone())
            .map_or_default(|tokens| syntax::parse::parse2::<MacroCalls>(tokens).map_or_default(|calls| calls.0))
    }

    fn banged(tokens: TokenStream) -> Option<TokenStream> {
        tokens.clone().into_iter().find_map(Self::bang).map(|_| tokens)
    }

    fn bang(tree: TokenTree) -> Option<Punct> {
        match tree {
            TokenTree::Punct(punct) => Some(punct).filter(|found| found.as_char() == Self::BANG),
            TokenTree::Group(group) => group.stream().into_iter().find_map(Self::bang),
            TokenTree::Ident(_) | TokenTree::Literal(_) => None,
        }
    }

    fn step(input: ParseStream) -> syntax::Result<Vec<Macro>> {
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

    fn skipped(tree: TokenTree) -> syntax::Result<Vec<Macro>> {
        match tree {
            TokenTree::Group(group) => syntax::parse::parse2::<MacroCalls>(group.stream()).map(|calls| calls.0),
            _ => Ok(Vec::new()),
        }
    }
}

impl Parse for MacroCalls {
    fn parse(input: ParseStream) -> syntax::Result<MacroCalls> {
        let steps = iter::from_fn(|| input.cursor().token_tree().map(|_| Self::step(input)));
        steps
            .collect::<syntax::Result<Vec<Vec<Macro>>>>()
            .map(|parsed| MacroCalls(parsed.into_iter().flatten().collect()))
    }
}
