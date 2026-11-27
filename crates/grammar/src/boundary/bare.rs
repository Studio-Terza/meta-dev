use logos::Logos;

#[derive(Logos, Clone, Debug, PartialEq, Eq)]
#[logos(crate = logos)]
enum Lexeme {
    #[regex(r"[A-Za-z0-9_][A-Za-z0-9_-]*")]
    Bare,
}

#[derive(Debug)]
pub struct Bare;

impl Bare {
    #[must_use]
    pub fn word(text: &str) -> Option<&str> {
        let mut lexer = Lexeme::lexer(text);
        let whole = lexer.next().filter(|lexed| *lexed == Ok(Lexeme::Bare)).map(|_bare| lexer.span());
        whole.filter(|span| *span == (0..text.len())).map(|_whole| text)
    }
}
