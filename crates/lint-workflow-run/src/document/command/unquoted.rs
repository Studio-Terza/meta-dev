use std::fmt;
use std::str::FromStr;

use crate::document::text::Text;

pub(super) mod hidden;
mod quote;
use hidden::Hidden;
use quote::Quote;

#[derive(Clone, Debug)]
pub(super) struct Unquoted(String);

impl Unquoted {
    pub(super) fn new(text: Text<'_>, hidden: Hidden) -> Unquoted {
        Unquoted(
            text.rendered()
                .chars()
                .scan(Quote::Outside, |quote, letter| {
                    let before = *quote;
                    *quote = before.after(letter);
                    Some(hidden.keeps(before, *quote).map(|()| letter))
                })
                .flatten()
                .collect(),
        )
    }

    pub(super) fn parsed<Parsed: FromStr>(&self) -> Result<Parsed, Parsed::Err> {
        self.to_string().parse()
    }

    pub(super) fn first_of(&self, symbols: &[&'static str]) -> Option<&'static str> {
        let shown = self.to_string();
        symbols.iter().copied().find(|symbol| shown.contains(*symbol))
    }
}

impl fmt::Display for Unquoted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
