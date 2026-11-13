use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};

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
        tokens.into_iter().fold(Flattening(Vec::new()), Flattening::gathered).0
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

    fn marks(delimiter: Delimiter) -> Option<Brackets> {
        match delimiter {
            Delimiter::Parenthesis => Some(Brackets { open: Mark::OPEN_PAREN, close: Mark::CLOSE_PAREN }),
            Delimiter::Brace => Some(Brackets { open: Mark::OPEN_BRACE, close: Mark::CLOSE_BRACE }),
            Delimiter::Bracket => Some(Brackets { open: Mark::OPEN_BRACKET, close: Mark::CLOSE_BRACKET }),
            Delimiter::None => None,
        }
    }

    fn marked<'raw>(mark: Mark, position: Position, equivalence: Equivalence) -> Option<ShapeToken<'raw>> {
        Some(mark)
            .filter(|candidate| *candidate == Mark::MINUS)
            .filter(|_| position == Position::BeforeLiteral)
            .filter(|_| equivalence.erases(Erased::Literals).is_some())
            .map_or(Some(ShapeToken::Mark(mark)), |_| None)
    }
}

#[derive(Clone, Copy, Debug)]
struct Brackets {
    open: Mark,
    close: Mark,
}

#[derive(Debug)]
struct Flattening(Vec<RawToken>);

impl Flattening {
    fn gathered(self, tree: TokenTree) -> Flattening {
        match tree {
            TokenTree::Literal(literal) => self.pushed(RawToken::Literal(Spelling::from(&literal))),
            TokenTree::Punct(punct) => self.pushed(RawToken::Mark(Mark::new(punct.as_char()))),
            TokenTree::Ident(ident) => self.pushed(RawToken::Word(Name::from(&ident))),
            TokenTree::Group(group) => self.grouped(&group),
        }
    }

    fn pushed(mut self, token: RawToken) -> Flattening {
        self.0.push(token);
        self
    }

    fn grouped(self, group: &Group) -> Flattening {
        let marks = RawToken::marks(group.delimiter());
        let opened = marks.into_iter().fold(self, |flat, brackets| flat.pushed(RawToken::Mark(brackets.open)));
        let inner = group.stream().into_iter().fold(opened, Flattening::gathered);
        marks.into_iter().fold(inner, |flat, brackets| flat.pushed(RawToken::Mark(brackets.close)))
    }
}
