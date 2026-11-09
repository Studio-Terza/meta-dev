use logos::{Lexer, Logos};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Piece {
    #[regex("[^\n]+", allow_greedy = true)]
    Text,
    #[token("\n")]
    Break,
}

#[derive(Debug)]
struct Broken<'text>(Lexer<'text, Piece>);

#[derive(Debug)]
pub struct Lines<'text>(Broken<'text>);

#[derive(Debug)]
pub struct KeptLines<'text>(Broken<'text>);

#[derive(Clone, Copy, Debug)]
struct Line<'text> {
    text: &'text str,
    kept: &'text str,
}

impl Piece {
    fn text<'text>(self, lexer: &mut Lexer<'text, Piece>) -> &'text str {
        match self {
            Piece::Break => "",
            Piece::Text => Piece::broken(lexer),
        }
    }

    fn broken<'text>(lexer: &mut Lexer<'text, Piece>) -> &'text str {
        let text = lexer.slice();
        lexer.next().map_or(text, |_break| text)
    }
}

impl<'text> Broken<'text> {
    fn new(text: &'text str) -> Broken<'text> {
        Broken(Piece::lexer(text))
    }

    fn line(&mut self) -> Option<Line<'text>> {
        let piece = self.0.next()?.unwrap_or(Piece::Text);
        let start = self.0.span().start;
        let text = piece.text(&mut self.0);
        let kept = self.0.source().get(start..self.0.span().end).unwrap_or_default();
        Some(Line { text, kept })
    }
}

impl<'text> From<&'text str> for Lines<'text> {
    fn from(text: &'text str) -> Lines<'text> {
        Lines(Broken::new(text))
    }
}

impl<'text> From<&'text str> for KeptLines<'text> {
    fn from(text: &'text str) -> KeptLines<'text> {
        KeptLines(Broken::new(text))
    }
}

impl<'text> Iterator for Lines<'text> {
    type Item = &'text str;

    fn next(&mut self) -> Option<&'text str> {
        self.0.line().map(|line| line.text)
    }
}

impl<'text> Iterator for KeptLines<'text> {
    type Item = &'text str;

    fn next(&mut self) -> Option<&'text str> {
        self.0.line().map(|line| line.kept)
    }
}
