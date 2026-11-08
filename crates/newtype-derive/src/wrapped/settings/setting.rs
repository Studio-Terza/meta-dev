use syn::parse::{Parse, ParseStream};
use syn::{Expr, Token};

use super::Checking;

mod keyword {
    syn::custom_keyword!(checked);
    syn::custom_keyword!(strategy);
}

pub enum Setting {
    Checked,
    Strategy(Box<Expr>),
}

impl Setting {
    pub fn checking(&self) -> Option<Checking> {
        match self {
            Setting::Checked => Some(Checking::Checked),
            Setting::Strategy(_) => None,
        }
    }

    pub fn strategy(&self) -> Option<&Expr> {
        match self {
            Setting::Strategy(strategy) => Some(strategy.as_ref()),
            Setting::Checked => None,
        }
    }
}

impl Parse for Setting {
    fn parse(input: ParseStream<'_>) -> syn::Result<Setting> {
        let look = input.lookahead1();
        let key = [(look.peek(keyword::checked), Key::Checked), (look.peek(keyword::strategy), Key::Strategy)]
            .into_iter()
            .find(|(peeked, _)| *peeked)
            .map(|(_, key)| key);
        key.map_or_else(|| Err(look.error()), |found| found.parsed(input))
    }
}

#[derive(Clone, Copy)]
enum Key {
    Checked,
    Strategy,
}

impl Key {
    fn parsed(self, input: ParseStream<'_>) -> syn::Result<Setting> {
        match self {
            Key::Checked => input.parse::<keyword::checked>().map(|_| Setting::Checked),
            Key::Strategy => input
                .parse::<keyword::strategy>()
                .and_then(|_| input.parse::<Token![=]>())
                .and_then(|_| input.parse())
                .map(|strategy| Setting::Strategy(Box::new(strategy))),
        }
    }
}
