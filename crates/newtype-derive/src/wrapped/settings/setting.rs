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
        if look.peek(keyword::check) {
            input.parse::<keyword::check>()?;
            input.parse::<Token![=]>()?;
            input.parse().map(Setting::Check)
        } else if look.peek(keyword::error) {
            input.parse::<keyword::error>()?;
            input.parse::<Token![=]>()?;
            input.parse().map(Setting::Error)
        } else if look.peek(keyword::strategy) {
            input.parse::<keyword::strategy>()?;
            input.parse::<Token![=]>()?;
            input.parse().map(Setting::Strategy)
        } else {
            Err(look.error())
        }
    }
}
