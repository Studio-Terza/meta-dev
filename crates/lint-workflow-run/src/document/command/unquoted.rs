use std::fmt;
use std::str::FromStr;

use newtype::Newtype;

use crate::excerpt::Text;
use crate::word::Word;

pub(super) mod hidden;
mod quote;
use hidden::Hidden;
use quote::Quote;

#[derive(Clone, Debug)]
pub(super) struct Unquoted(String);

impl Unquoted {
    pub(super) fn new(text: Text<'_>, hidden: Hidden) -> Unquoted {
        Unquoted(
            text.to_string()
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

    pub(super) fn first_of(&self, symbols: &[Word]) -> Option<Word> {
        let shown = self.to_string();
        symbols.iter().copied().find(|symbol| shown.contains(symbol.into_inner()))
    }
}

impl fmt::Display for Unquoted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
