use std::convert::Infallible;
use std::iter;
use std::str::FromStr;

use syntax::parse::Speculative;
use syntax::parse::{Parse, ParseBuffer, ParseStream};
use syntax::{Group, Ident, Literal, Punct, Span, TokenStream, TokenTree};
use syntax::{LitStr, Macro, Path, braced, bracketed, parenthesized, token};

use lint_layout::Position;

use crate::ignored::Ignored;
use crate::namespace::UsePath;
use invariants::Ignoring;

#[derive(Debug)]
pub(super) struct Spelled {
    path: UsePath,
    started_at: Position,
}

impl Spelled {
    pub(super) fn handed<Made>(self, to: impl FnOnce(UsePath, Position) -> Made) -> Made {
        to(self.path, self.started_at)
    }
}

#[derive(Debug)]
struct Respanned(Span);

impl Respanned {
    fn spanned(&self, tokens: TokenStream) -> TokenStream {
        tokens.into_iter().map(|token| self.respanned(token)).collect()
    }

    fn respanned(&self, token: TokenTree) -> TokenTree {
        match token {
            TokenTree::Group(group) => TokenTree::Group(Group::new(group.delimiter(), self.spanned(group.stream()))),
            TokenTree::Ident(ident) => self.placed(ident),
            TokenTree::Punct(_) | TokenTree::Literal(_) => token,
        }
    }

    fn placed(&self, ident: Ident) -> TokenTree {
        let mut placed = ident;
        placed.set_span(self.0);
        TokenTree::Ident(placed)
    }
}

#[derive(Debug, Default)]
struct Holes(Vec<TokenStream>);

impl Holes {
    const OPEN: char = '{';
    const CLOSE: char = '}';
    const OPENING: &str = "{";
    const CLOSING: &str = "}";
}

impl FromStr for Holes {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Holes, Infallible> {
        let braces: Vec<(usize, &str)> = text.match_indices([Holes::OPEN, Holes::CLOSE]).collect();
        let holes = braces
            .iter()
            .zip(braces.iter().skip(1))
            .filter(|((_, left), (_, right))| [*left, *right] == [Holes::OPENING, Holes::CLOSING])
            .filter_map(|((open, _), (close, _))| {
                text.get(open + 1..*close)?.parse::<TokenStream>().ignoring(Ignored::Unparsed)
            });
        Ok(Holes(holes.collect()))
    }
}

#[derive(Debug)]
struct Interpolated;

impl Interpolated {
    fn read(token: TokenTree) -> Vec<TokenStream> {
        match token {
            TokenTree::Group(group) => Self::stream(group.stream()),
            TokenTree::Literal(literal) => Self::holes(&literal),
            TokenTree::Ident(_) | TokenTree::Punct(_) => Vec::new(),
        }
    }

    fn stream(tokens: TokenStream) -> Vec<TokenStream> {
        tokens.into_iter().flat_map(Self::read).collect()
    }

    fn holes(literal: &Literal) -> Vec<TokenStream> {
        let text = syntax::parse::parse2::<LitStr>(TokenTree::Literal(literal.clone()).into())
            .ignoring(Ignored::Unparsed)
            .map_or_default(|found| found.value());
        let Ok(Holes(holes)) = text.parse::<Holes>();
        let spanned = Respanned(literal.span());
        holes.into_iter().map(|hole| spanned.spanned(hole)).collect()
    }
}

#[derive(Debug)]
pub(super) struct TokenPaths(Vec<Spelled>);

impl TokenPaths {
    const ACCESS: char = '.';

    pub(super) fn interpolated(invoked: &Macro) -> Vec<Spelled> {
        Interpolated::stream(invoked.tokens.clone()).into_iter().flat_map(Self::within).collect()
    }

    pub(super) fn within(tokens: TokenStream) -> Vec<Spelled> {
        syntax::parse::parse2::<TokenPaths>(tokens).ignoring(Ignored::Unparsed).map_or_default(|paths| paths.0)
    }

    fn attempted(input: ParseStream, attempt: &ParseBuffer) -> syntax::Result<Vec<Spelled>> {
        match attempt.call(Path::parse_mod_style) {
            Ok(path) => Ok(Self::spelled(&path)).inspect(|_| input.advance_to(attempt)),
            Err(_) => Bracketing::peeked(input).inside(input),
        }
    }

    fn spelled(path: &Path) -> Vec<Spelled> {
        let first = path.segments.first();
        let spelled = first
            .map(|segment| Spelled { path: UsePath::from(path), started_at: Position::from(segment.ident.span()) });
        spelled.into_iter().collect()
    }

    fn skipped(tree: TokenTree, input: ParseStream) -> syntax::Result<Vec<Spelled>> {
        match tree {
            TokenTree::Group(group) => Ok(Self::within(group.stream())),
            TokenTree::Punct(punct) => Self::punctuated(&punct, input),
            _ => Ok(Vec::new()),
        }
    }

    fn punctuated(punct: &Punct, input: ParseStream) -> syntax::Result<Vec<Spelled>> {
        Some(punct.as_char())
            .filter(|spelled| *spelled == Self::ACCESS)
            .map_or_else(|| Ok(Vec::new()), |_| input.parse::<Option<Ident>>().map(|_| Vec::new()))
    }
}

#[derive(Clone, Copy, Debug)]
enum Bracketing {
    Brace,
    Bracket,
    Paren,
    Loose,
}

impl Bracketing {
    fn peeked(input: ParseStream) -> Bracketing {
        input
            .peek(token::Brace)
            .then_some(Bracketing::Brace)
            .or_else(|| input.peek(token::Bracket).then_some(Bracketing::Bracket))
            .or_else(|| input.peek(token::Paren).then_some(Bracketing::Paren))
            .unwrap_or(Bracketing::Loose)
    }

    fn inside(self, input: ParseStream) -> syntax::Result<Vec<Spelled>> {
        match self {
            Bracketing::Brace => Braces::opened(input).and_then(|content| content.parse::<TokenPaths>()).map(Vec::from),
            Bracketing::Bracket => {
                Brackets::opened(input).and_then(|content| content.parse::<TokenPaths>()).map(Vec::from)
            }
            Bracketing::Paren => Parens::opened(input).and_then(|content| content.parse::<TokenPaths>()).map(Vec::from),
            Bracketing::Loose => TokenPaths::skipped(input.parse()?, input),
        }
    }
}

impl From<TokenPaths> for Vec<Spelled> {
    fn from(paths: TokenPaths) -> Vec<Spelled> {
        paths.0
    }
}

trait Opens {
    fn opened(input: ParseStream<'_>) -> syntax::Result<ParseBuffer<'_>>;
}

#[derive(Debug)]
struct Braces;

#[derive(Debug)]
struct Brackets;

#[derive(Debug)]
struct Parens;

impl Opens for Braces {
    fn opened(input: ParseStream<'_>) -> syntax::Result<ParseBuffer<'_>> {
        let content;
        braced!(content in input);
        Ok(content)
    }
}

impl Opens for Brackets {
    fn opened(input: ParseStream<'_>) -> syntax::Result<ParseBuffer<'_>> {
        let content;
        bracketed!(content in input);
        Ok(content)
    }
}

impl Opens for Parens {
    fn opened(input: ParseStream<'_>) -> syntax::Result<ParseBuffer<'_>> {
        let content;
        parenthesized!(content in input);
        Ok(content)
    }
}

impl Parse for TokenPaths {
    fn parse(input: ParseStream) -> syntax::Result<TokenPaths> {
        let steps = iter::from_fn(|| input.cursor().token_tree().map(|_| Self::attempted(input, &input.fork())));
        steps
            .collect::<syntax::Result<Vec<Vec<Spelled>>>>()
            .map(|parsed| TokenPaths(parsed.into_iter().flatten().collect()))
    }
}
