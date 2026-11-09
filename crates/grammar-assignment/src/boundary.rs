use std::str::FromStr;

use grammar::logos::Logos;
use newtype::{Arbitrary, Checked, Newtype};

use grammar_unspelled::Unspelled;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Spelling<'text> {
    #[regex("[A-Za-z_][A-Za-z0-9_]*", |lexer| lexer.slice())]
    Name(&'text str),
    #[token("=")]
    Assign,
}

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(checked, strategy = "[A-Za-z_][A-Za-z0-9_]{0,12}")]
pub struct Emptied(String);

impl<'text> Spelling<'text> {
    fn emptied([name, assign]: [Spelling<'text>; 2]) -> Option<&'text str> {
        Some(assign).filter(|spelling| *spelling == Spelling::Assign)?;
        name.name()
    }

    fn name(self) -> Option<&'text str> {
        match self {
            Spelling::Name(name) => Some(name),
            Spelling::Assign => None,
        }
    }
}

impl Checked for Emptied {
    type Error = Unspelled;

    fn checked(name: &String) -> Result<&String, Unspelled> {
        let spelled: Vec<Result<Spelling<'_>, ()>> = Spelling::lexer(name).collect();
        <[Result<Spelling<'_>, ()>; 1]>::try_from(spelled)
            .ok()
            .and_then(|[only]| only.ok())
            .and_then(Spelling::name)
            .map(|_name| name)
            .ok_or(Unspelled)
    }
}

impl FromStr for Emptied {
    type Err = Unspelled;

    fn from_str(line: &str) -> Result<Emptied, Unspelled> {
        let spelled: Result<Vec<Spelling<'_>>, ()> = Spelling::lexer(line).collect();
        spelled
            .ok()
            .and_then(|lexed| <[Spelling<'_>; 2]>::try_from(lexed).ok())
            .and_then(Spelling::emptied)
            .ok_or(Unspelled)
            .and_then(|name| Emptied::try_new(name.to_owned()))
    }
}
