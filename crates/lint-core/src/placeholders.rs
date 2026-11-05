use std::convert::Infallible;
use std::str::FromStr;

use proc_macro2::TokenStream;

#[derive(Debug)]
pub struct Placeholders(Vec<TokenStream>);

impl Placeholders {
    const ESCAPED_PAIR: usize = 2;

    #[must_use]
    pub fn streams(self) -> Vec<TokenStream> {
        self.0
    }
}

impl FromStr for Placeholders {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Placeholders, Infallible> {
        let braces: Vec<usize> = text.char_indices().filter(|(_, symbol)| *symbol == '{').map(|(at, _)| at).collect();
        Ok(Placeholders(
            braces
                .chunk_by(|first, second| *second == *first + 1)
                .filter(|run| run.len() % Self::ESCAPED_PAIR == 1)
                .filter_map(|run| run.last())
                .filter_map(|opening| text.get(opening + 1..))
                .filter_map(|rest| rest.match_indices('}').next().and_then(|(close, _)| rest.get(..close)))
                .filter_map(|expression| expression.parse().ok())
                .collect(),
        ))
    }
}
