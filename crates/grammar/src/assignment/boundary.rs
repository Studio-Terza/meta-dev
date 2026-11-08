use logos::Logos;
use newtype::Newtype;

use crate::unspelled::Unspelled;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Spelling<'text> {
    #[regex("[A-Za-z_][A-Za-z0-9_]*", |lexer| lexer.slice())]
    Name(&'text str),
    #[token("=")]
    Assign,
}

#[derive(Newtype, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Emptied<'text>(&'text str);

impl<'text> Emptied<'text> {
    fn spelled([name, assign]: [Spelling<'text>; 2]) -> Option<Emptied<'text>> {
        Some(assign).filter(|spelling| *spelling == Spelling::Assign)?;
        name.name().map(Emptied)
    }
}

impl<'text> Spelling<'text> {
    fn name(self) -> Option<&'text str> {
        match self {
            Spelling::Name(name) => Some(name),
            Spelling::Assign => None,
        }
    }
}

impl<'text> TryFrom<&'text str> for Emptied<'text> {
    type Error = Unspelled;

    fn try_from(line: &'text str) -> Result<Emptied<'text>, Unspelled> {
        let spelled: Result<Vec<Spelling<'text>>, ()> = Spelling::lexer(line).collect();
        spelled
            .ok()
            .and_then(|lexed| <[Spelling<'text>; 2]>::try_from(lexed).ok())
            .and_then(Emptied::spelled)
            .ok_or(Unspelled)
    }
}
