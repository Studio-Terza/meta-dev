use super::lexeme::{Lexeme, Mark};

#[derive(Clone, Copy, Debug)]
pub(super) struct Transcriber<'source>(&'source [Lexeme]);

impl<'source> Transcriber<'source> {
    pub(super) fn lexemes(self) -> &'source [Lexeme] {
        self.0
    }
}

#[derive(Debug)]
pub(super) struct Arms;

impl Arms {
    const SHAPES: [&'static [Mark]; 2] = [
        &[Mark::Group, Mark::Equals, Mark::Arrow, Mark::Group, Mark::Semicolon],
        &[Mark::Group, Mark::Equals, Mark::Arrow, Mark::Group],
    ];
    const WIDEST: usize = 5;
    const TRANSCRIBER: usize = 3;

    pub(super) fn transcribed(lexemes: &[Lexeme]) -> Option<Vec<Transcriber<'_>>> {
        lexemes
            .chunks(Self::WIDEST)
            .map(|arm| {
                Self::SHAPES
                    .iter()
                    .find(|shape| arm.iter().map(Lexeme::mark).eq(shape.iter().copied()))
                    .and_then(|_| arm.get(Self::TRANSCRIBER))
                    .and_then(Lexeme::children)
                    .map(Transcriber)
            })
            .collect()
    }
}
