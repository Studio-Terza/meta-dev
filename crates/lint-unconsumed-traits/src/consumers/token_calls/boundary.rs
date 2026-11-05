use std::convert::Infallible;
use std::str::FromStr;

use proc_macro2::TokenStream;

#[derive(Debug)]
pub(super) struct Placeholders(Vec<TokenStream>);

impl Placeholders {
    pub(super) fn streams(self) -> Vec<TokenStream> {
        self.0
    }

    fn expression(text: &str, opening: usize) -> Option<&str> {
        text.get(opening + 1..).and_then(|rest| {
            rest.char_indices().find(|(_, symbol)| *symbol == '}').and_then(|(length, _)| rest.get(..length))
        })
    }
}

impl FromStr for Placeholders {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Placeholders, Infallible> {
        let braces: Vec<usize> = text.char_indices().filter(|(_, symbol)| *symbol == '{').map(|(at, _)| at).collect();
        Ok(Placeholders(
            braces
                .chunk_by(|first, second| *second == *first + 1)
                .filter(|run| run.len() % 2 == 1)
                .filter_map(|run| run.last())
                .filter_map(|opening| Self::expression(text, *opening))
                .filter_map(|expression| expression.parse().ok())
                .collect(),
        ))
    }
}
