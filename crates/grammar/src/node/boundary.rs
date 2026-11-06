use logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Spelling {
    #[regex("n[0-9]+", |lexer| lexer.slice().get(1..).and_then(|digits| digits.parse::<i64>().ok()))]
    Numbered(i64),
}

#[derive(Debug)]
pub struct Ids;

impl Ids {
    #[must_use]
    pub fn numbered(text: &str) -> Option<i64> {
        let spelled: Vec<Result<Spelling, ()>> = Spelling::lexer(text).collect();
        <[Result<Spelling, ()>; 1]>::try_from(spelled).ok().and_then(|[only]| only.ok()).map(Spelling::number)
    }
}

impl Spelling {
    fn number(self) -> i64 {
        match self {
            Spelling::Numbered(number) => number,
        }
    }
}
