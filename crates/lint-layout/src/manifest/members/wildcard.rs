use std::convert::Infallible;
use std::str::FromStr;

use crate::matching::Matching;
use crate::name::Name;
use crate::word::Word;
use newtype::Newtype;

#[derive(Debug)]
pub struct Wildcard(Vec<char>);

impl Wildcard {
    const ANY: char = '*';

    fn agrees(pattern: &[char], text: &[char]) -> bool {
        match pattern.split_first() {
            None => text.is_empty(),
            Some((&Self::ANY, rest)) => {
                Self::agrees(rest, text) || text.split_first().is_some_and(|(_, tail)| Self::agrees(pattern, tail))
            }
            Some((letter, rest)) => {
                text.split_first().is_some_and(|(first, tail)| first == letter && Self::agrees(rest, tail))
            }
        }
    }
}

impl FromStr for Wildcard {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Wildcard, Infallible> {
        Ok(Wildcard(text.chars().collect()))
    }
}

impl Matching for Wildcard {
    fn holds(name: &Name, pattern: Word<'_>) -> bool {
        let letters = pattern.into_inner().parse::<Wildcard>();
        let spelled = name.spelled().parse::<Wildcard>();
        letters.iter().zip(&spelled).any(|(wanted, found)| Self::agrees(&wanted.0, &found.0))
    }
}
