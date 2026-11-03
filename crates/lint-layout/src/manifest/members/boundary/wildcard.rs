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

    fn agrees<'text>(pattern: &[char], text: &'text [char]) -> Option<&'text [char]> {
        match pattern.split_first() {
            None => text.is_empty().then_some(text),
            Some((&Self::ANY, rest)) => {
                Self::agrees(rest, text).or_else(|| Self::agrees(pattern, text.get(1..)?)).map(|_| text)
            }
            Some((letter, rest)) => text
                .split_first()
                .filter(|(first, _)| *first == letter)
                .and_then(|(_, tail)| Self::agrees(rest, tail))
                .map(|_| text),
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
    fn holds<'word>(name: &Name, pattern: Word<'word>) -> Option<Word<'word>> {
        let letters = pattern.into_inner().parse::<Wildcard>();
        let spelled = name.to_string().parse::<Wildcard>();
        letters.iter().zip(&spelled).find_map(|(wanted, found)| Self::agrees(&wanted.0, &found.0)).map(|_| pattern)
    }
}
