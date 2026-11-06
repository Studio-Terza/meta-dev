use logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Letter {
    #[regex(r"[A-Za-z0-9]+")]
    Kept,
    #[regex(r"[^A-Za-z0-9]")]
    Replaced,
}

#[derive(Debug)]
pub struct Letters;

impl Letters {
    pub fn slugged(text: &str) -> String {
        Letter::lexer(text)
            .spanned()
            .map(|(letter, range)| Letter::spelled(letter, text.get(range).unwrap_or_default()))
            .collect()
    }
}

impl Letter {
    fn spelled(letter: Result<Letter, ()>, written: &str) -> &str {
        match letter {
            Ok(Letter::Kept) => written,
            Ok(Letter::Replaced) | Err(()) => "-",
        }
    }
}
