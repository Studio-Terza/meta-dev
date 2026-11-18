use std::fmt;

use syntax::{Ident, LitStr};

use super::super::case::Case;

pub(super) struct Spelled<'variant> {
    variant: &'variant Ident,
    case: Case,
}

impl Spelled<'_> {
    pub(super) fn spelled(variant: &Ident, case: Case) -> LitStr {
        LitStr::new(&Spelled { variant, case }.to_string(), variant.span())
    }
}

impl fmt::Display for Spelled<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let written = self.variant.to_string();
        let separator = Self::separator(self.case);
        let first = written.chars().take(1).map(|letter| letter.to_ascii_lowercase());
        let rest = written.chars().skip(1).flat_map(|letter| {
            letter.is_ascii_uppercase().then_some(separator).into_iter().chain([letter.to_ascii_lowercase()])
        });
        formatter.write_str(&first.chain(rest).collect::<String>())
    }
}

impl Spelled<'_> {
    const fn separator(case: Case) -> char {
        match case {
            Case::Snake => '_',
            Case::Kebab => '-',
        }
    }
}
