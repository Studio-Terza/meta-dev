use std::fmt;
use std::num::NonZeroU16;
use std::str::FromStr;

use logos::Logos;

use crate::unspelled::Unspelled;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Spelling {
    #[regex("[0-9]+", |lexer| lexer.slice().parse().ok())]
    Number(u16),
    #[token("/")]
    Slash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sharding {
    index: u16,
    count: NonZeroU16,
}

impl Sharding {
    const SEPARATOR: char = '/';

    fn spelled([index, slash, count]: [Spelling; 3]) -> Option<Sharding> {
        Some(slash).filter(|spelling| *spelling == Spelling::Slash)?;
        let nonzero = NonZeroU16::new(count.number()?)?;
        index.number().filter(|below| *below < nonzero.get()).map(|number| Sharding { index: number, count: nonzero })
    }
}

impl Spelling {
    fn number(self) -> Option<u16> {
        match self {
            Spelling::Number(number) => Some(number),
            Spelling::Slash => None,
        }
    }
}

impl FromStr for Sharding {
    type Err = Unspelled;

    fn from_str(text: &str) -> Result<Sharding, Unspelled> {
        let spelled: Result<Vec<Spelling>, ()> = Spelling::lexer(text).collect();
        spelled
            .ok()
            .and_then(|lexed| <[Spelling; 3]>::try_from(lexed).ok())
            .and_then(Sharding::spelled)
            .ok_or(Unspelled)
    }
}

impl fmt::Display for Sharding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}{}", self.index, Self::SEPARATOR, self.count)
    }
}
