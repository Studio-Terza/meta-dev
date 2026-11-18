mod expansion;
mod only_field;

use syntax::TokenStream;
use syntax::{DeriveInput, Generics, Ident};

use crate::settings::Settings;
pub use expansion::{CheckedDeserializing, Constructors, Deserializing, Expansion, Formatting, Serializing, Strategy};
use only_field::OnlyField;
use syntax::Compiled;

pub struct Wrapped {
    ident: Ident,
    generics: Generics,
    field: OnlyField,
    settings: Settings,
}

impl Wrapped {
    pub fn expanded<Derived: Expansion>(tokens: TokenStream) -> TokenStream {
        syntax::parse::parse2(tokens).map(Self::derived::<Derived>).compiled()
    }

    pub fn derived<Derived: Expansion>(input: DeriveInput) -> TokenStream {
        Self::split(input).map(Derived::of).compiled()
    }

    fn split(DeriveInput { attrs, vis, ident, generics, data }: DeriveInput) -> syntax::Result<Wrapped> {
        match OnlyField::taken(data) {
            Ok(field) => Settings::declared(&attrs).map(|settings| Wrapped { ident, generics, field, settings }),
            Err(refused) => Settings::declared(&attrs)
                .and_then(|_| Err(OnlyField::refused(DeriveInput { attrs, vis, ident, generics, data: refused }))),
        }
    }
}
