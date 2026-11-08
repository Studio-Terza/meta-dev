mod expansion;
mod only_field;
mod settings;

use proc_macro2::TokenStream;
use syn::{DeriveInput, Generics, Ident};

pub use expansion::{CheckedDeserializing, Constructors, Deserializing, Expansion, Formatting, Serializing, Strategy};
use only_field::OnlyField;
use settings::Settings;

pub struct Wrapped {
    ident: Ident,
    generics: Generics,
    field: OnlyField,
    settings: Settings,
}

impl Wrapped {
    pub fn expanded<Derived: Expansion>(tokens: TokenStream) -> TokenStream {
        syn::parse2(tokens).and_then(Self::split).map_or_else(|error| error.to_compile_error(), Derived::of)
    }

    fn split(DeriveInput { attrs, vis, ident, generics, data }: DeriveInput) -> syn::Result<Wrapped> {
        match OnlyField::taken(data) {
            Ok(field) => Settings::declared(&attrs).map(|settings| Wrapped { ident, generics, field, settings }),
            Err(refused) => Settings::declared(&attrs)
                .and_then(|_| Err(OnlyField::refused(DeriveInput { attrs, vis, ident, generics, data: refused }))),
        }
    }
}
