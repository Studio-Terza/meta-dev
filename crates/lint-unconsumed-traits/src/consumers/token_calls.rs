use std::iter;

use proc_macro2::{Delimiter, Group, Ident, Literal, TokenStream, TokenTree};
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{AngleBracketedGenericArguments, ExprPath, LitStr, Macro, Path, Token};

use lint_core::Placeholders;
use lint_layout::Name;

#[derive(Debug)]
pub(crate) struct TokenCalls(Vec<Name>);

#[derive(Debug)]
struct MethodCall(Name);

#[derive(Debug)]
struct PathCall(Name);

struct Arguments;

struct Turbofish;

impl Turbofish {
    fn skip(attempt: ParseStream) -> syn::Result<()> {
        match attempt.parse::<Option<Token![::]>>()? {
            Some(_) => AngleBracketedGenericArguments::parse(attempt).map(drop),
            None => Ok(()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Text {
    Plain,
    Formatted,
}

#[derive(Debug)]
enum Step {
    Call(Name),
    Group(Group),
    Nested(Text, Group),
    Text(Literal),
    Skip,
}

struct Interpolated(LitStr);

impl Arguments {
    const NOT_A_CALL: &'static str = "not a call";

    fn follow(attempt: ParseStream) -> syn::Result<()> {
        attempt
            .fork()
            .parse::<Group>()
            .map_err(|_| attempt.error(Self::NOT_A_CALL))
            .and_then(|arguments| Self::opens(arguments.delimiter(), attempt))
    }

    fn opens(delimiter: Delimiter, attempt: ParseStream) -> syn::Result<()> {
        match delimiter {
            Delimiter::Parenthesis => Ok(()),
            Delimiter::Brace | Delimiter::Bracket | Delimiter::None => Err(attempt.error(Self::NOT_A_CALL)),
        }
    }
}

impl Parse for MethodCall {
    fn parse(attempt: ParseStream) -> syn::Result<MethodCall> {
        attempt.parse::<Token![.]>()?;
        let method: Ident = attempt.parse()?;
        Turbofish::skip(attempt)?;
        Arguments::follow(attempt).map(|()| MethodCall(Name::from(&method)))
    }
}

impl Parse for PathCall {
    fn parse(attempt: ParseStream) -> syn::Result<PathCall> {
        let path: ExprPath = attempt.parse()?;
        let segments = &path.path.segments;
        segments
            .iter()
            .rev()
            .zip(segments.iter().rev().skip(1))
            .next()
            .map(|(segment, _)| PathCall(Name::from(&segment.ident)))
            .ok_or_else(|| attempt.error(Arguments::NOT_A_CALL))
    }
}

impl From<MethodCall> for Name {
    fn from(call: MethodCall) -> Name {
        call.0
    }
}

impl From<PathCall> for Name {
    fn from(call: PathCall) -> Name {
        call.0
    }
}

impl Text {
    const FORMATTING: [&'static str; 1] = ["rsx"];

    fn named(name: &Ident) -> Text {
        match Self::FORMATTING.into_iter().find(|candidate| name == candidate) {
            Some(_) => Text::Formatted,
            None => Text::Plain,
        }
    }

    fn classify(invocation: &Macro) -> Text {
        invocation.path.segments.last().map_or(Text::Plain, |segment| Self::named(&segment.ident))
    }

    fn interpolated(self, literal: Literal) -> Vec<Name> {
        match self {
            Text::Formatted => Interpolated::called_in(literal),
            Text::Plain => Vec::new(),
        }
    }
}

impl Step {
    fn taking<Call: Parse + Into<Name>>(input: ParseStream) -> syn::Result<Step> {
        let attempt = input.fork();
        attempt.parse::<Call>().map(|call| {
            input.advance_to(&attempt);
            Step::Call(call.into())
        })
    }
}

struct Nested(Text, Group);

impl Parse for Nested {
    fn parse(input: ParseStream) -> syn::Result<Nested> {
        let attempt = input.fork();
        let path = Path::parse_mod_style(&attempt)?;
        attempt.parse::<Token![!]>()?;
        let group: Group = attempt.parse()?;
        input.advance_to(&attempt);
        let text = path.segments.last().map_or(Text::Plain, |segment| Text::named(&segment.ident));
        Ok(Nested(text, group))
    }
}

impl From<Nested> for Step {
    fn from(nested: Nested) -> Step {
        Step::Nested(nested.0, nested.1)
    }
}

impl Parse for Step {
    fn parse(input: ParseStream) -> syn::Result<Step> {
        input
            .parse::<Token![..]>()
            .map(|_| Step::Skip)
            .or_else(|_| Self::taking::<MethodCall>(input))
            .or_else(|_| input.parse::<Nested>().map(Step::from))
            .or_else(|_| Self::taking::<PathCall>(input))
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

impl Step {
    fn names(self, text: Text) -> Vec<Name> {
        match self {
            Step::Call(method) => vec![method],
            Step::Group(group) => TokenCalls::within(group.stream(), text),
            Step::Nested(own, group) => TokenCalls::within(group.stream(), own),
            Step::Text(literal) => text.interpolated(literal),
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
            .flat_map(|stream| TokenCalls::within(stream, Text::Plain))
            .collect()
    }
}

impl TokenCalls {
    pub(crate) fn called_in(invocation: &Macro) -> Vec<Name> {
        let text = Text::classify(invocation);
        invocation
            .parse_body_with(move |input: ParseStream| Self::scan(input, text))
            .map(|calls| calls.0)
            .unwrap_or_default()
    }

    fn within(tokens: TokenStream, text: Text) -> Vec<Name> {
        (move |input: ParseStream| Self::scan(input, text)).parse2(tokens).map(|calls| calls.0).unwrap_or_default()
    }

    fn scan(input: ParseStream, text: Text) -> syn::Result<TokenCalls> {
        let steps = iter::from_fn(|| input.cursor().token_tree().map(|_| input.parse::<Step>()));
        steps
            .map(|step| step.map(|step| step.names(text)))
            .collect::<syn::Result<Vec<Vec<Name>>>>()
            .map(|steps| TokenCalls(steps.into_iter().flatten().collect()))
    }
}
