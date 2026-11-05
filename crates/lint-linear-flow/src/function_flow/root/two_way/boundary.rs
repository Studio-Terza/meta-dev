use std::convert::Infallible;
use std::str::FromStr;

use syn::Ident;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Casing {
    Variant,
    Other,
}

impl FromStr for Casing {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Casing, Infallible> {
        Ok(text
            .chars()
            .next()
            .filter(|first| first.is_uppercase())
            .and_then(|_| text.chars().find(|letter| letter.is_lowercase()))
            .map_or(Casing::Other, |_| Casing::Variant))
    }
}

impl From<&Ident> for Casing {
    fn from(name: &Ident) -> Casing {
        name.to_string().parse().unwrap_or(Casing::Other)
    }
}
