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
    pub const fn function(&self) -> Option<&Path> {
        match self {
            Self::Check(function) => Some(function),
            Self::Error(_) | Self::Strategy(_) => None,
        }
    }

    pub const fn error(&self) -> Option<&Type> {
        match self {
            Self::Error(error) => Some(error),
            Self::Check(_) | Self::Strategy(_) => None,
        }
    }

    pub const fn strategy(&self) -> Option<&Expr> {
        match self {
            Self::Strategy(strategy) => Some(strategy),
            Self::Check(_) | Self::Error(_) => None,
        }
    }
}

impl Parse for Setting {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let look = input.lookahead1();
        let key = [
            (look.peek(keyword::check), Key::Check),
            (look.peek(keyword::error), Key::Error),
            (look.peek(keyword::strategy), Key::Strategy),
        ]
        .into_iter()
        .find(|(peeked, _)| *peeked)
        .map(|(_, key)| key);
        key.map_or_else(|| Err(look.error()), |key| key.parsed(input))
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
            Self::Check => input
                .parse::<keyword::check>()
                .and_then(|_| input.parse::<Token![=]>())
                .and_then(|_| input.parse())
                .map(Setting::Check),
            Self::Error => input
                .parse::<keyword::error>()
                .and_then(|_| input.parse::<Token![=]>())
                .and_then(|_| input.parse())
                .map(Setting::Error),
            Self::Strategy => input
                .parse::<keyword::strategy>()
                .and_then(|_| input.parse::<Token![=]>())
                .and_then(|_| input.parse())
                .map(Setting::Strategy),
        }
    }
}
