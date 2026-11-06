use super::Lexeme;
use super::boundary::Token;

#[derive(Debug)]
pub struct Lexemes;

impl Lexemes {
    #[must_use]
    pub fn of(text: &impl AsRef<str>) -> Vec<Lexeme> {
        Token::lexed(text.as_ref())
    }
}
