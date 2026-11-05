use std::convert::Infallible;
use std::str::FromStr;

use proc_macro2::TokenStream;

#[derive(Debug)]
pub(super) struct Placeholders(Vec<TokenStream>);

impl Placeholders {
    pub(super) fn streams(self) -> Vec<TokenStream> {
        self.0
    }

    fn placeholder(text: &str, at: usize) -> Option<&str> {
        let before = text.get(..at).map_or(0, |head| head.chars().rev().take_while(|symbol| *symbol == '{').count());
        let after = text.get(at + 1..).and_then(|tail| tail.chars().next());
        (before % 2 == 0)
            .then_some(at)
            .filter(|_| after != Some('{'))
            .and_then(|opening| text.get(opening + 1..))
            .and_then(|rest| {
                rest.char_indices().find(|(_, symbol)| *symbol == '}').and_then(|(length, _)| rest.get(..length))
            })
    }
}

impl FromStr for Placeholders {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Placeholders, Infallible> {
        Ok(Placeholders(
            text.char_indices()
                .filter(|(_, symbol)| *symbol == '{')
                .filter_map(|(at, _)| Self::placeholder(text, at))
                .filter_map(|expression| expression.parse().ok())
                .collect(),
        ))
    }
}
