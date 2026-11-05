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
        let screaming = Some(text)
            .filter(|name| name.contains('_'))
            .filter(|name| name.chars().all(|letter| matches!(letter, 'A'..='Z' | '0'..='9' | '_')));
        Ok(text
            .chars()
            .next()
            .filter(|first| first.is_uppercase())
            .and(screaming.map_or(Some(Casing::Variant), |_| None))
            .unwrap_or(Casing::Other))
    }
}

impl From<&Ident> for Casing {
    fn from(name: &Ident) -> Casing {
        name.to_string().parse().unwrap_or(Casing::Other)
    }
}
