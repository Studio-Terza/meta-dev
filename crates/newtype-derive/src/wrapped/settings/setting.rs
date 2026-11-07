use syn::parse::{Parse, ParseStream};
use syn::{Expr, Path, Token, Type};

mod keyword {
    syn::custom_keyword!(check);
    syn::custom_keyword!(error);
    syn::custom_keyword!(strategy);
}

pub enum Setting {
    Check(Path),
    Error(Type),
    Strategy(Expr),
}

impl Setting {
    pub fn function(&self) -> Option<&Path> {
        match self {
            Setting::Check(function) => Some(function),
            Setting::Error(_) | Setting::Strategy(_) => None,
        }
    }

    pub fn error(&self) -> Option<&Type> {
        match self {
            Setting::Error(error) => Some(error),
            Setting::Check(_) | Setting::Strategy(_) => None,
        }
    }

    pub fn strategy(&self) -> Option<&Expr> {
        match self {
            Setting::Strategy(strategy) => Some(strategy),
            Setting::Check(_) | Setting::Error(_) => None,
        }
    }
}

impl Parse for Setting {
    fn parse(input: ParseStream<'_>) -> syn::Result<Setting> {
        let look = input.lookahead1();
        let key = [
            (look.peek(keyword::check), Key::Check),
            (look.peek(keyword::error), Key::Error),
            (look.peek(keyword::strategy), Key::Strategy),
        ]
        .into_iter()
        .find(|(peeked, _)| *peeked)
        .map(|(_, key)| key);
        key.map_or_else(|| Err(look.error()), |found| found.parsed(input))
    }
}

#[derive(Clone, Copy)]
enum Key {
    Check,
    Error,
    Strategy,
}

impl Key {
    fn parsed(self, input: ParseStream<'_>) -> syn::Result<Setting> {
        match self {
            Key::Check => input
                .parse::<keyword::check>()
                .and_then(|_| input.parse::<Token![=]>())
                .and_then(|_| input.parse())
                .map(Setting::Check),
            Key::Error => input
                .parse::<keyword::error>()
                .and_then(|_| input.parse::<Token![=]>())
                .and_then(|_| input.parse())
                .map(Setting::Error),
            Key::Strategy => input
                .parse::<keyword::strategy>()
                .and_then(|_| input.parse::<Token![=]>())
                .and_then(|_| input.parse())
                .map(Setting::Strategy),
        }
    }
}
