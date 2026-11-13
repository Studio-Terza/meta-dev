use std::convert::Infallible;
use std::str::FromStr;

use proc_macro2::{Delimiter, Group, Ident, Literal, Span, TokenStream, TokenTree};
use syn::{LitStr, Macro};

use lint_layout::{Position, TokenPaths as Parsed};

use crate::namespace::UsePath;

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
            .filter_map(|((open, _), (close, _))| text.get(open + 1..*close)?.parse::<TokenStream>().ok());
        Ok(Holes(holes.collect()))
    }
}

#[derive(Debug)]
struct Interpolated;

impl Interpolated {
    fn read(token: TokenTree) -> TokenStream {
        match token {
            TokenTree::Group(group) => Self::stream(group.stream()),
            TokenTree::Literal(literal) => Self::holes(&literal),
            TokenTree::Ident(_) | TokenTree::Punct(_) => TokenStream::new(),
        }
    }

    fn stream(tokens: TokenStream) -> TokenStream {
        tokens.into_iter().map(Self::read).collect()
    }

    fn holes(literal: &Literal) -> TokenStream {
        let text =
            syn::parse2::<LitStr>(TokenTree::Literal(literal.clone()).into()).map_or_default(|found| found.value());
        let Ok(Holes(holes)) = text.parse::<Holes>();
        let spanned = Respanned(literal.span());
        holes
            .into_iter()
            .map(|hole| TokenTree::Group(Group::new(Delimiter::Parenthesis, spanned.spanned(hole))))
            .collect()
    }
}

#[derive(Debug)]
pub(super) struct TokenPaths;

impl TokenPaths {
    pub(super) fn interpolated(invoked: &Macro) -> Vec<Spelled> {
        Self::within(Interpolated::stream(invoked.tokens.clone()))
    }

    pub(super) fn within(tokens: TokenStream) -> Vec<Spelled> {
        let parsed = Parsed::within(tokens);
        let spelled = parsed.iter().filter_map(|path| {
            let first = path.segments.first()?;
            Some(Spelled { path: UsePath::from(path), started_at: Position::from(first.ident.span()) })
        });
        spelled.collect()
    }
}
