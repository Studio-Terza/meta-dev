use syn::parse::{Parse, ParseStream};
use syn::{Expr, Token};

use super::Checking;

mod keyword {
    syn::custom_keyword!(checked);
    syn::custom_keyword!(strategy);
    syn::custom_keyword!(from);
    syn::custom_keyword!(generate);
}

pub(crate) enum Setting {
    Checked,
    Expressed(Expressed, Box<Expr>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Expressed {
    Strategy,
    From,
    Generate,
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
        let key = [
            (look.peek(keyword::checked), Key::Checked),
            (look.peek(keyword::strategy), Key::Expressed(Expressed::Strategy)),
            (look.peek(keyword::from), Key::Expressed(Expressed::From)),
            (look.peek(keyword::generate), Key::Expressed(Expressed::Generate)),
        ]
        .into_iter()
        .find(|(peeked, _)| *peeked)
        .map(|(_, key)| key);
        key.map_or_else(|| Err(look.error()), |found| found.parsed(input))
    }
}

#[derive(Clone, Copy)]
enum Key {
    Checked,
    Expressed(Expressed),
}

impl Key {
    fn parsed(self, input: ParseStream<'_>) -> syn::Result<Setting> {
        match self {
            Key::Checked => input.parse::<keyword::checked>().map(|_| Setting::Checked),
            Key::Expressed(expressed) => expressed
                .keyword(input)
                .and_then(|()| input.parse::<Token![=]>())
                .and_then(|_| input.parse())
                .map(|expression| Setting::Expressed(expressed, Box::new(expression))),
        }
    }
}

impl Expressed {
    fn keyword(self, input: ParseStream<'_>) -> syn::Result<()> {
        match self {
            Expressed::Strategy => input.parse::<keyword::strategy>().map(|_| ()),
            Expressed::From => input.parse::<keyword::from>().map(|_| ()),
            Expressed::Generate => input.parse::<keyword::generate>().map(|_| ()),
        }
    }
}
