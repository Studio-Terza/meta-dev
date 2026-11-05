mod boundary;

use std::iter;

use proc_macro2::{Delimiter, Group, Ident, Literal, TokenStream, TokenTree};
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{AngleBracketedGenericArguments, ExprPath, LitStr, Macro, Token};

use lint_layout::Name;

use boundary::Placeholders;

#[derive(Debug)]
pub(crate) struct TokenCalls(Vec<Name>);

#[derive(Debug)]
struct MethodCall(Name);

#[derive(Debug)]
struct PathCall(Name);

struct Arguments;

#[derive(Clone, Copy, Debug)]
enum Text {
    Plain,
    Formatted,
}

#[derive(Debug)]
enum Step {
    Call(Name),
    Group(Group),
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
            .ok()
            .filter(|arguments| arguments.delimiter() == Delimiter::Parenthesis)
            .map(|_| ())
            .ok_or_else(|| attempt.error(Self::NOT_A_CALL))
    }
}

impl Parse for MethodCall {
    fn parse(attempt: ParseStream) -> syn::Result<MethodCall> {
        attempt.parse::<Token![.]>()?;
        let method: Ident = attempt.parse()?;
        attempt.peek(Token![::]).then(|| AngleBracketedGenericArguments::parse_turbofish(attempt)).transpose()?;
        Arguments::follow(attempt).map(|()| MethodCall(Name::from(&method)))
    }
}

impl Parse for PathCall {
    fn parse(attempt: ParseStream) -> syn::Result<PathCall> {
        let path: ExprPath = attempt.parse()?;
        let name = path
            .path
            .segments
            .iter()
            .nth(1)
            .and_then(|_| path.path.segments.last())
            .map(|segment| Name::from(&segment.ident))
            .ok_or_else(|| attempt.error(Arguments::NOT_A_CALL))?;
        Arguments::follow(attempt).map(|()| PathCall(name))
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
    const FORMATTING: [&'static str; 22] = [
        "rsx",
        "format",
        "format_args",
        "print",
        "println",
        "eprint",
        "eprintln",
        "write",
        "writeln",
        "panic",
        "todo",
        "unimplemented",
        "unreachable",
        "assert",
        "assert_eq",
        "assert_ne",
        "debug_assert",
        "trace",
        "debug",
        "info",
        "warn",
        "error",
    ];

    fn classify(invocation: &Macro) -> Text {
        invocation
            .path
            .segments
            .last()
            .filter(|segment| Self::FORMATTING.iter().any(|name| segment.ident == name))
            .map_or(Text::Plain, |_| Text::Formatted)
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

impl Parse for Step {
    fn parse(input: ParseStream) -> syn::Result<Step> {
        input
            .parse::<Token![..]>()
            .map(|_| Step::Skip)
            .or_else(|_| Self::taking::<MethodCall>(input))
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
            .flat_map(|stream| TokenCalls::within(stream, Text::Formatted))
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
