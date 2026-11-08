use logos::Logos;
use newtype::Newtype;

use crate::unspelled::Unspelled;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(skip r"\s+")]
enum Spelling<'text> {
    #[regex(r"\S+", |lexer| lexer.slice())]
    Word(&'text str),
}

#[derive(Newtype, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Program<'text>(&'text str);

impl<'text> TryFrom<&'text str> for Program<'text> {
    type Error = Unspelled;

    fn try_from(command: &'text str) -> Result<Program<'text>, Unspelled> {
        Spelling::lexer(command).next().and_then(Result::ok).map(|Spelling::Word(word)| Program(word)).ok_or(Unspelled)
    }
}
