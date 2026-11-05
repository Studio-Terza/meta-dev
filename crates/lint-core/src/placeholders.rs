use std::convert::Infallible;
use std::str::FromStr;

use proc_macro2::TokenStream;

#[derive(Debug)]
pub struct Placeholders(Vec<Argument>);

#[derive(Debug)]
struct Argument(Vec<TokenStream>);

#[derive(Debug)]
struct Unclosed;

impl Argument {
    fn streams(self) -> Vec<TokenStream> {
        self.0
    }
}

impl FromStr for Argument {
    type Err = Unclosed;

    fn from_str(rest: &str) -> Result<Argument, Unclosed> {
        let colons: Vec<usize> = rest.match_indices(':').map(|(at, _)| at).collect();
        let single = colons
            .chunk_by(|first, second| *second == *first + 1)
            .find(|run| run.len() == 1)
            .and_then(|run| run.first());
        let close = rest.match_indices('}').next().map(|(at, _)| at);
        let colon = single.copied().zip(close).filter(|(at, end)| at < end).map(|(at, _)| at);
        let widths = colon
            .zip(close)
            .and_then(|(at, end)| rest.get(at + 1..end))
            .into_iter()
            .flat_map(|spec| spec.match_indices('$').filter_map(move |(at, _)| spec.get(..at)))
            .filter_map(|before| before.parse::<TokenStream>().ok())
            .filter_map(|tokens| tokens.into_iter().last())
            .map(TokenStream::from);
        colon
            .or(close)
            .and_then(|end| rest.get(..end))
            .and_then(|text| text.parse::<TokenStream>().ok())
            .map(|stream| Argument(std::iter::once(stream).chain(widths).collect()))
            .ok_or(Unclosed)
    }
}

impl Placeholders {
    const ESCAPED_PAIR: usize = 2;

    #[must_use]
    pub fn streams(self) -> Vec<TokenStream> {
        self.0.into_iter().flat_map(Argument::streams).collect()
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
                .filter_map(|rest| rest.parse().ok())
                .collect(),
        ))
    }
}
