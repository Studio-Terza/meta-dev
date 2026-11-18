use std::fmt;

use syntax::{Ident, LitStr};

pub(super) struct Snake<'variant>(&'variant Ident);

impl Snake<'_> {
    const BREAK: char = '_';

    pub(super) fn spelled(variant: &Ident) -> LitStr {
        LitStr::new(&Snake(variant).to_string(), variant.span())
    }
}

impl fmt::Display for Snake<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let written = self.0.to_string();
        let first = written.chars().take(1).map(|letter| letter.to_ascii_lowercase());
        let rest = written.chars().skip(1).flat_map(|letter| {
            letter.is_ascii_uppercase().then_some(Self::BREAK).into_iter().chain([letter.to_ascii_lowercase()])
        });
        formatter.write_str(&first.chain(rest).collect::<String>())
    }
}
