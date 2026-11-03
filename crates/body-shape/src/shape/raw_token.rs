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

    pub(crate) fn shaped(&self, position: Position, slots: &mut Slots) -> Option<ShapeToken> {
        match self {
            RawToken::Literal => Some(ShapeToken::Literal),
            RawToken::Mark(Mark::MINUS) if position == Position::BeforeLiteral => None,
            RawToken::Mark(mark) => Some(ShapeToken::Mark(*mark)),
            RawToken::Word(name) => Some(slots.classified(name, position)),
        }
    }

    fn of_tree(tree: TokenTree) -> Vec<RawToken> {
        match tree {
            TokenTree::Literal(_) => vec![RawToken::Literal],
            TokenTree::Punct(punct) => vec![RawToken::Mark(Mark::new(punct.as_char()))],
            TokenTree::Ident(ident) => vec![RawToken::Word(Name::from(&ident))],
            TokenTree::Group(group) => {
                let inner = Self::flattened(group.stream());
                match group.delimiter() {
                    Delimiter::Parenthesis => Self::between(Mark::OPEN_PAREN, inner, Mark::CLOSE_PAREN),
                    Delimiter::Brace => Self::between(Mark::OPEN_BRACE, inner, Mark::CLOSE_BRACE),
                    Delimiter::Bracket => Self::between(Mark::OPEN_BRACKET, inner, Mark::CLOSE_BRACKET),
                    Delimiter::None => inner,
                }
            }
        }
    }

    fn between(open: Mark, inner: Vec<RawToken>, close: Mark) -> Vec<RawToken> {
        [RawToken::Mark(open)].into_iter().chain(inner).chain([RawToken::Mark(close)]).collect()
    }
}
