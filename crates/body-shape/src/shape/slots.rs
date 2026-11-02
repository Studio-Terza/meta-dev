use lint_layout::{Casing, Name, Whole, Word, words};

use super::ShapeToken;
use super::position::Position;
use super::slot::Slot;

#[derive(Debug, Default)]
pub(crate) struct Slots(Vec<Name>);

impl Slots {
    const LITERAL_WORDS: [Word<'static>; 2] = words!["true", "false"];
    const KEYWORDS: [Word<'static>; 40] = words![
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "fn", "for",
        "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static",
        "struct", "super", "trait", "type", "unsafe", "use", "where", "while", "bool", "str", "char", "u8", "usize"
    ];

    pub(crate) fn classified(&mut self, name: &Name, position: Position) -> ShapeToken {
        match (name.casing(), position) {
            _ if name.matches::<Whole>(&Self::LITERAL_WORDS) => ShapeToken::Literal,
            _ if name.matches::<Whole>(&Self::KEYWORDS) => ShapeToken::Word(name.clone()),
            (Casing::Lower, Position::Named) => ShapeToken::Word(name.clone()),
            (Casing::Lower, _) => ShapeToken::Local(self.numbered(name)),
            _ => ShapeToken::Word(name.clone()),
        }
    }

    fn numbered(&mut self, name: &Name) -> Slot {
        let known = self.0.iter().position(|seen| seen == name);
        Slot::new(known.unwrap_or_else(|| {
            let next = self.0.len();
            self.0.push(name.clone());
            next
        }))
    }
}
