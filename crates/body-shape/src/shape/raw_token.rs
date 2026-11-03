use proc_macro2::{Delimiter, TokenStream, TokenTree};

use lint_layout::Name;

use super::ShapeToken;
use super::mark::Mark;
use super::position::Position;
use super::slots::Slots;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RawToken {
    Word(Name),
    Literal,
    Mark(Mark),
}

impl RawToken {
    pub(crate) fn flattened(tokens: TokenStream) -> Vec<RawToken> {
        tokens.into_iter().flat_map(Self::of_tree).collect()
    }

    pub(crate) fn shaped<'raw>(&'raw self, position: Position, slots: &mut Slots<'raw>) -> Option<ShapeToken<'raw>> {
        match self {
            RawToken::Literal => Some(ShapeToken::Literal),
            RawToken::Mark(mark) => Self::marked(*mark, position),
            RawToken::Word(name) => Some(slots.classified(name, position)),
        }
    }

    fn of_tree(tree: TokenTree) -> Vec<RawToken> {
        match tree {
            TokenTree::Literal(_) => vec![RawToken::Literal],
            TokenTree::Punct(punct) => vec![RawToken::Mark(Mark::new(punct.as_char()))],
            TokenTree::Ident(ident) => vec![RawToken::Word(Name::from(&ident))],
            TokenTree::Group(group) => Self::delimited(group.delimiter(), Self::flattened(group.stream())),
        }
    }

    fn delimited(delimiter: Delimiter, inner: Vec<RawToken>) -> Vec<RawToken> {
        match delimiter {
            Delimiter::Parenthesis => Self::between(Mark::OPEN_PAREN, inner, Mark::CLOSE_PAREN),
            Delimiter::Brace => Self::between(Mark::OPEN_BRACE, inner, Mark::CLOSE_BRACE),
            Delimiter::Bracket => Self::between(Mark::OPEN_BRACKET, inner, Mark::CLOSE_BRACKET),
            Delimiter::None => inner,
        }
    }

    fn marked<'raw>(mark: Mark, position: Position) -> Option<ShapeToken<'raw>> {
        Some(mark)
            .filter(|mark| *mark == Mark::MINUS)
            .filter(|_| position == Position::BeforeLiteral)
            .map_or(Some(ShapeToken::Mark(mark)), |_| None)
    }

    fn between(open: Mark, inner: Vec<RawToken>, close: Mark) -> Vec<RawToken> {
        [RawToken::Mark(open)].into_iter().chain(inner).chain([RawToken::Mark(close)]).collect()
    }
}
