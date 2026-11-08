use proc_macro2::{Delimiter, TokenStream, TokenTree};

use equivalence::{Equivalence, Erased};
use lint_layout::Name;

use super::ShapeToken;
use super::mark::Mark;
use super::position::Position;
use super::slots::Slots;
use super::spelling::{Sign, Spelled, Spelling, Spoken};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RawToken {
    Word(Name),
    Literal(Spelling),
    Mark(Mark),
}

impl RawToken {
    pub(crate) fn flattened(tokens: TokenStream) -> Vec<RawToken> {
        tokens.into_iter().flat_map(Self::of_tree).collect()
    }

    pub(crate) fn shaped<'raw>(
        &'raw self,
        position: Position,
        slots: &mut Slots<'raw>,
        equivalence: Equivalence,
    ) -> Option<ShapeToken<'raw>> {
        match self {
            RawToken::Literal(spelling) => Some(spelling.shaped(equivalence)),
            RawToken::Mark(mark) => Self::marked(*mark, position, equivalence),
            RawToken::Word(name) => Some(slots.classified(name, position, equivalence)),
        }
    }

    pub(crate) fn spelled(self, sign: Sign) -> Option<Spelled> {
        match self {
            RawToken::Literal(spelling) => Some(Spelled::new(sign, Spoken::Literal(spelling))),
            RawToken::Word(name) => Slots::literal(&name).is_some().then(|| Spelled::new(sign, Spoken::Word(name))),
            RawToken::Mark(_) => None,
        }
    }

    fn of_tree(tree: TokenTree) -> Vec<RawToken> {
        match tree {
            TokenTree::Literal(literal) => vec![RawToken::Literal(Spelling::new(literal.to_string()))],
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

    fn marked<'raw>(mark: Mark, position: Position, equivalence: Equivalence) -> Option<ShapeToken<'raw>> {
        Some(mark)
            .filter(|candidate| *candidate == Mark::MINUS)
            .filter(|_| position == Position::BeforeLiteral)
            .filter(|_| equivalence.erases(Erased::Literals).is_some())
            .map_or(Some(ShapeToken::Mark(mark)), |_| None)
    }

    fn between(open: Mark, inner: Vec<RawToken>, close: Mark) -> Vec<RawToken> {
        [RawToken::Mark(open)].into_iter().chain(inner).chain([RawToken::Mark(close)]).collect()
    }
}
