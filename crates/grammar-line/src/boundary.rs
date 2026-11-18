use grammar::logos::Logos;
use grammar::{Ignoring, Unlexed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Spelling {
    #[regex(
        r"[^\P{White_Space}\n\x0B\x0C\r\x{85}\x{2028}\x{2029}]*[^\p{White_Space}][^\n\x0B\x0C\r\x{85}\x{2028}\x{2029}]*",
        allow_greedy = true
    )]
    Line,
}

#[derive(Debug)]
pub struct Unbroken;

impl Unbroken {
    #[must_use]
    pub fn spelled(text: &str) -> Option<&str> {
        let spelled: Vec<Result<Spelling, ()>> = Spelling::lexer(text).collect();
        <[Result<Spelling, ()>; 1]>::try_from(spelled)
            .ignoring(Unlexed::OtherShape)
            .and_then(|[only]| only.ignoring(Unlexed::NotThisGrammar))
            .map(|_line| text)
    }
}
