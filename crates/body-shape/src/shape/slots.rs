use lint_layout::{Casing, Name, Whole, Word, words};

use super::ShapeToken;
use super::position::Position;
use super::slot::Slot;

#[derive(Debug, Default)]
pub(crate) struct Slots<'raw>(Vec<&'raw Name>);

impl<'raw> Slots<'raw> {
    const LITERAL_WORDS: [Word<'static>; 2] = words!["true", "false"];
    const KEYWORDS: [Word<'static>; 40] = words![
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "fn", "for",
        "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static",
        "struct", "super", "trait", "type", "unsafe", "use", "where", "while", "bool", "str", "char", "u8", "usize"
    ];

    pub(crate) fn classified(&mut self, name: &'raw Name, position: Position) -> ShapeToken<'raw> {
        match (name.casing(), position) {
            _ if name.matches::<Whole>(&Self::LITERAL_WORDS).is_some() => ShapeToken::Literal,
            _ if name.matches::<Whole>(&Self::KEYWORDS).is_some() => ShapeToken::Word(name),
            (Casing::Lower, Position::Named) => ShapeToken::Word(name),
            (Casing::Lower, _) => ShapeToken::Local(self.numbered(name)),
            _ => ShapeToken::Word(name),
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
