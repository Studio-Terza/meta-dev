mod check;
mod expansion;
mod only_field;
mod settings;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Generics, Ident};

use check::Check;
pub use expansion::{Constructors, Expansion, Formatting, Strategy};
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

    fn unchecked(name: &Ident, generics: &Generics, field: &OnlyField) -> TokenStream {
        let (implemented, arguments, bounds) = generics.split_for_impl();
        let member = field.member();
        let inner = field.inner();
        quote! {
                    impl #implemented #name #arguments #bounds {
                        #[must_use]
                        pub const fn new(inner: #inner) -> Self {
                            Self { #member: inner }
                        }
                    }

                    impl #implemented ::newtype::Newtype for #name #arguments #bounds {
                        type Inner = #inner;

                        fn new(inner: Self::Inner) -> Self {
                            Self { #member: inner }
                        }

                        fn into_inner(self) -> Self::Inner {
                            self.#member
                        }
                    }
        }
    }

    fn checked(name: &Ident, generics: &Generics, field: &OnlyField, check: &Check) -> TokenStream {
        let (implemented, arguments, bounds) = generics.split_for_impl();
        let member = field.member();
        let inner = field.inner();
        let function = check.function();
        let error = check.error();
        quote! {
                    impl #implemented #name #arguments #bounds {
                        pub fn try_new(inner: #inner) -> ::core::result::Result<Self, #error> {
                            #function(&inner)?;
                            ::core::result::Result::Ok(Self { #member: inner })
                        }

                        #[must_use]
                        pub fn into_inner(self) -> #inner {
                            self.#member
                        }
                    }
        }
    }

    fn plain_strategy() -> TokenStream {
        quote! {
            ::newtype::proptest::strategy::Strategy::prop_map(inner, <Self as ::newtype::Newtype>::new)
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
