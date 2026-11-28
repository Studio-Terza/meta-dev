#![deny(clippy::wildcard_enum_match_arm)]

pub mod bare;
pub mod detail;
pub mod git;
pub mod letters;
pub mod lines;
pub mod notice;
pub mod prose;
pub mod report;
pub mod repository;
pub mod shell;
pub mod spec_line;
pub mod test_log;
mod unlexed;
mod unspelled;

use invariants::Ignoring;
use logos::Logos;

use unlexed::Unlexed;
pub use unspelled::Unspelled;

pub trait Grammar<'text>: Sized {
    fn read(text: &'text str) -> Option<Self>;
}

#[derive(Debug)]
struct Whole;

impl Whole {
    fn tokens<'text, Token: Logos<'text, Source = str, Extras = ()>>(text: &'text str) -> Option<Vec<Token>> {
        Token::lexer(text).collect::<Result<Vec<Token>, Token::Error>>().ignoring(Unlexed::NotThisGrammar)
    }

    fn exactly<'text, Token: Logos<'text, Source = str, Extras = ()>, const COUNT: usize>(
        text: &'text str,
    ) -> Option<[Token; COUNT]> {
        <[Token; COUNT]>::try_from(Whole::tokens(text)?).ignoring(Unlexed::OtherShape)
    }
}
