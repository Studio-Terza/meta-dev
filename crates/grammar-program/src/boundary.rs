use std::str::FromStr;

use grammar::logos::Logos;
use newtype::{Arbitrary, Checked, Newtype};

use grammar_unspelled::Unspelled;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
#[logos(skip r"\s+")]
enum Spelling<'text> {
    #[regex(r"\S+", |lexer| lexer.slice())]
    Word(&'text str),
}

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(checked, strategy = "[!-~]{1,12}")]
pub struct Program(String);

impl Checked for Program {
    type Error = Unspelled;

    fn checked(word: &String) -> Result<&String, Unspelled> {
        let spelled: Vec<Result<Spelling<'_>, ()>> = Spelling::lexer(word).collect();
        <[Result<Spelling<'_>, ()>; 1]>::try_from(spelled)
            .ok()
            .and_then(|[only]| only.ok())
            .filter(|Spelling::Word(lexed)| *lexed == word.as_str())
            .map(|_word| word)
            .ok_or(Unspelled)
    }
}

impl FromStr for Program {
    type Err = Unspelled;

    fn from_str(command: &str) -> Result<Program, Unspelled> {
        Spelling::lexer(command)
            .next()
            .and_then(Result::ok)
            .ok_or(Unspelled)
            .and_then(|Spelling::Word(word)| Program::try_new(word.to_owned()))
    }
}
