mod boundary;

use std::iter;

use proc_macro2::{Delimiter, Group, Ident, Literal, TokenStream, TokenTree};
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};
use syn::{AngleBracketedGenericArguments, LitStr, Macro, Token};

use lint_layout::Name;

use boundary::Placeholders;

#[derive(Debug)]
pub(crate) struct TokenCalls(Vec<Name>);

#[derive(Debug)]
struct Called(Name);

#[derive(Debug)]
enum Step {
    Call(Name),
    Group(Group),
    Text(Literal),
    Skip,
}

struct Interpolated(LitStr);

impl Called {
    const NOT_A_CALL: &'static str = "not a method call";
}

impl Parse for Called {
    fn parse(attempt: ParseStream) -> syn::Result<Called> {
        attempt.parse::<Token![.]>()?;
        let method: Ident = attempt.parse()?;
        attempt.peek(Token![::]).then(|| AngleBracketedGenericArguments::parse_turbofish(attempt)).transpose()?;
        attempt
            .fork()
            .parse::<Group>()
            .ok()
            .filter(|arguments| arguments.delimiter() == Delimiter::Parenthesis)
            .map(|_| Called(Name::from(&method)))
            .ok_or_else(|| attempt.error(Self::NOT_A_CALL))
    }
}

impl Step {
    fn called(input: ParseStream) -> syn::Result<Step> {
        let attempt = input.fork();
        attempt.parse::<Called>().map(|Called(method)| {
            input.advance_to(&attempt);
            Step::Call(method)
        })
    }
}

impl Parse for Step {
    fn parse(input: ParseStream) -> syn::Result<Step> {
        input
            .parse::<Token![..]>()
            .map(|_| Step::Skip)
            .or_else(|_| Self::called(input))
            .or_else(|_| input.parse::<TokenTree>().map(Step::from))
    }
}

impl From<TokenTree> for Step {
    fn from(tree: TokenTree) -> Step {
        match tree {
            TokenTree::Group(group) => Step::Group(group),
            TokenTree::Literal(literal) => Step::Text(literal),
            TokenTree::Ident(_) | TokenTree::Punct(_) => Step::Skip,
        }
    }
}

impl From<Step> for Vec<Name> {
    fn from(step: Step) -> Vec<Name> {
        match step {
            Step::Call(method) => vec![method],
            Step::Group(group) => TokenCalls::within(group.stream()),
            Step::Text(literal) => Interpolated::called_in(literal),
            Step::Skip => Vec::new(),
        }
    }
}

impl Interpolated {
    fn called_in(literal: Literal) -> Vec<Name> {
        syn::parse2::<LitStr>(TokenTree::Literal(literal).into()).ok().map_or_else(Vec::new, |text| Self(text).names())
    }

    fn names(&self) -> Vec<Name> {
        self.0
            .value()
            .parse::<Placeholders>()
            .into_iter()
            .flat_map(Placeholders::streams)
            .flat_map(TokenCalls::within)
            .collect()
    }
}

impl TokenCalls {
    pub(crate) fn called_in(invocation: &Macro) -> Vec<Name> {
        invocation.parse_body::<TokenCalls>().map(|calls| calls.0).unwrap_or_default()
    }

    fn within(tokens: TokenStream) -> Vec<Name> {
        syn::parse2::<TokenCalls>(tokens).map(|calls| calls.0).unwrap_or_default()
    }
}

impl Parse for TokenCalls {
    fn parse(input: ParseStream) -> syn::Result<TokenCalls> {
        let steps = iter::from_fn(|| input.cursor().token_tree().map(|_| input.parse::<Step>()));
        steps
            .map(|step| step.map(Vec::<Name>::from))
            .collect::<syn::Result<Vec<Vec<Name>>>>()
            .map(|steps| TokenCalls(steps.into_iter().flatten().collect()))
    }
}
