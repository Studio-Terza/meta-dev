use syn::parse::{Parse, ParseStream};
use syn::{Expr, Token};

use super::Checking;

mod keyword {
    syn::custom_keyword!(checked);
    syn::custom_keyword!(strategy);
}

pub(crate) enum Setting {
    Checked,
    Expressed(Expressed, Box<Expr>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Expressed {
    Strategy,
}

impl Setting {
    pub(crate) fn checking(&self) -> Option<Checking> {
        match self {
            Setting::Checked => Some(Checking::Checked),
            Setting::Expressed(..) => None,
        }
    }

    pub(crate) fn expressed(&self, wanted: Expressed) -> Option<&Expr> {
        match self {
            Setting::Expressed(named, expression) => (*named == wanted).then_some(expression.as_ref()),
            Setting::Checked => None,
        }
    }
}

impl Parse for Setting {
    fn parse(input: ParseStream<'_>) -> syn::Result<Setting> {
        let look = input.lookahead1();
        if look.peek(keyword::checked) {
            input.parse::<keyword::checked>().map(|_| Setting::Checked)
        } else if look.peek(keyword::strategy) {
            input.parse::<keyword::strategy>()?;
            input.parse::<Token![=]>()?;
            input.parse().map(|expression| Setting::Expressed(Expressed::Strategy, Box::new(expression)))
        } else {
            Err(look.error())
        }
    }
}
