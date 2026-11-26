use grammar::logos::Logos;
use grammar::{Ignoring, Unlexed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Spelling {
    #[regex("[A-Za-z0-9][A-Za-z0-9._-]*")]
    Name,
}

#[derive(Debug)]
pub struct Names;

impl Names {
    #[must_use]
    pub fn named(text: &str) -> Option<&str> {
        let spelled: Vec<Result<Spelling, ()>> = Spelling::lexer(text).collect();
        <[Result<Spelling, ()>; 1]>::try_from(spelled)
            .ignoring(Unlexed::OtherShape)
            .and_then(|[only]| only.ignoring(Unlexed::NotThisGrammar))
            .map(|_name| text)
    }
}
