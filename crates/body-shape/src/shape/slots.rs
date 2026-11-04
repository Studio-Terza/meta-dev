mod renaming;

use equivalence::{Equivalence, Erased};
use lint_layout::{Name, Whole, Word, words};

use super::ShapeToken;
use super::position::Position;
use super::slot::Slot;
use renaming::Renaming;

#[derive(Debug, Default)]
pub(crate) struct Slots<'raw>(Vec<&'raw Name>);

impl<'raw> Slots<'raw> {
    const LITERAL_WORDS: [Word<'static>; 2] = words!["true", "false"];
    const KEYWORDS: [Word<'static>; 40] = words![
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "fn", "for",
        "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static",
        "struct", "super", "trait", "type", "unsafe", "use", "where", "while", "bool", "str", "char", "u8", "usize"
    ];

    pub(crate) fn classified(
        &mut self,
        name: &'raw Name,
        position: Position,
        equivalence: Equivalence,
    ) -> ShapeToken<'raw> {
        Self::literal(name)
            .map(|_| Self::literal_word(name, equivalence))
            .or_else(|| name.matches::<Whole>(&Self::KEYWORDS).map(|_| ShapeToken::Word(name)))
            .unwrap_or_else(|| self.renamed(name, Renaming::chosen(name, position, equivalence)))
    }

    pub(crate) fn literal(name: &Name) -> Option<&Name> {
        name.matches::<Whole>(&Self::LITERAL_WORDS).map(|_| name)
    }

    fn literal_word(name: &'raw Name, equivalence: Equivalence) -> ShapeToken<'raw> {
        equivalence.erases(Erased::Literals).map_or(ShapeToken::Word(name), |_| ShapeToken::Literal)
    }

    fn renamed(&mut self, name: &'raw Name, renaming: Renaming) -> ShapeToken<'raw> {
        match renaming {
            Renaming::Slotted => ShapeToken::Local(self.numbered(name)),
            Renaming::Kept => ShapeToken::Word(name),
        }
    }

    fn numbered(&mut self, name: &'raw Name) -> Slot {
        let known = self.0.iter().position(|seen| *seen == name);
        Slot::new(known.unwrap_or_else(|| {
            let next = self.0.len();
            self.0.push(name);
            next
        }))
    }
}
