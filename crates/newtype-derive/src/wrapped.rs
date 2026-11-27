mod expansion;
mod only_field;

use proc_macro2::TokenStream;
use syn::{DeriveInput, Generics, Ident};

use crate::settings::Settings;
pub(crate) use expansion::{CheckedDeserializing, Constructors, Deserializing, Expansion, Formatting, Serializing, Strategy};
use only_field::OnlyField;

pub(crate) struct Wrapped {
    ident: Ident,
    generics: Generics,
    field: OnlyField,
    settings: Settings,
}

impl Wrapped {
    pub(crate) fn expanded<Derived: Expansion>(tokens: TokenStream) -> TokenStream {
        match syn::parse2(tokens).map(Self::derived::<Derived>) {
            Ok(expansion) => expansion,
            Err(error) => error.to_compile_error(),
        }
    }

    pub(crate) fn derived<Derived: Expansion>(input: DeriveInput) -> TokenStream {
        match Self::split(input).map(Derived::of) {
            Ok(expansion) => expansion,
            Err(error) => error.to_compile_error(),
        }
    }

    fn split(DeriveInput { attrs, vis, ident, generics, data }: DeriveInput) -> syn::Result<Wrapped> {
        match OnlyField::taken(data) {
            Ok(field) => Settings::declared(&attrs).map(|settings| Wrapped { ident, generics, field, settings }),
            Err(refused) => Settings::declared(&attrs)
                .and_then(|_| Err(OnlyField::refused(DeriveInput { attrs, vis, ident, generics, data: refused }))),
        }
    }
}
