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
    pub(crate) fn flattened(tokens: TokenStream) -> Vec<Self> {
        tokens.into_iter().flat_map(Self::of_tree).collect()
    }

    pub(crate) fn shaped<'raw>(
        &'raw self,
        position: Position,
        slots: &mut Slots<'raw>,
        equivalence: Equivalence,
    ) -> Option<ShapeToken<'raw>> {
        match self {
            Self::Literal(spelling) => Some(spelling.shaped(equivalence)),
            Self::Mark(mark) => Self::marked(*mark, position, equivalence),
            Self::Word(name) => Some(slots.classified(name, position, equivalence)),
        }
    }

    pub(crate) fn spelled(self, sign: Sign) -> Option<Spelled> {
        match self {
            Self::Literal(spelling) => Some(Spelled::new(sign, Spoken::Literal(spelling))),
            Self::Word(name) => Slots::literal(&name).is_some().then(|| Spelled::new(sign, Spoken::Word(name))),
            Self::Mark(_) => None,
        }
    }

    fn of_tree(tree: TokenTree) -> Vec<Self> {
        match tree {
            TokenTree::Literal(literal) => vec![Self::Literal(Spelling::new(literal.to_string()))],
            TokenTree::Punct(punct) => vec![Self::Mark(Mark::new(punct.as_char()))],
            TokenTree::Ident(ident) => vec![Self::Word(Name::from(&ident))],
            TokenTree::Group(group) => Self::delimited(group.delimiter(), Self::flattened(group.stream())),
        }
    }

    fn delimited(delimiter: Delimiter, inner: Vec<Self>) -> Vec<Self> {
        match delimiter {
            Delimiter::Parenthesis => Self::between(Mark::OPEN_PAREN, inner, Mark::CLOSE_PAREN),
            Delimiter::Brace => Self::between(Mark::OPEN_BRACE, inner, Mark::CLOSE_BRACE),
            Delimiter::Bracket => Self::between(Mark::OPEN_BRACKET, inner, Mark::CLOSE_BRACKET),
            Delimiter::None => inner,
        }
    }

    fn marked<'raw>(mark: Mark, position: Position, equivalence: Equivalence) -> Option<ShapeToken<'raw>> {
        Some(mark)
            .filter(|mark| *mark == Mark::MINUS)
            .filter(|_| position == Position::BeforeLiteral)
            .filter(|_| equivalence.erases(Erased::Literals).is_some())
            .map_or(Some(ShapeToken::Mark(mark)), |_| None)
    }

    fn between(open: Mark, inner: Vec<Self>, close: Mark) -> Vec<Self> {
        [Self::Mark(open)].into_iter().chain(inner).chain([Self::Mark(close)]).collect()
    }
}
