use std::convert::Infallible;
use std::str::FromStr;

use proc_macro2::TokenStream;

#[derive(Debug)]
pub(super) struct Placeholders(Vec<TokenStream>);

impl Placeholders {
    pub(super) fn streams(self) -> Vec<TokenStream> {
        self.0
    }

    fn enclosed(text: &str, start: usize) -> Option<&str> {
        text.get(start..).and_then(|rest| {
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
                .filter_map(|(at, _)| Self::enclosed(text, at + 1))
                .filter_map(|expression| expression.parse().ok())
                .collect(),
        ))
    }
}
