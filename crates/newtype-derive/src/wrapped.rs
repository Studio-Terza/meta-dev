mod check;
mod only_field;
mod settings;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Generics, Ident, parse_quote};

use check::Check;
use only_field::OnlyField;
use settings::Settings;

pub struct Wrapped {
    ident: Ident,
    generics: Generics,
    field: OnlyField,
    settings: Settings,
}

impl Wrapped {
    pub fn expanded(tokens: TokenStream, expansion: fn(Wrapped) -> TokenStream) -> TokenStream {
        syn::parse2(tokens).and_then(Self::split).map_or_else(|error| error.to_compile_error(), expansion)
    }

    pub fn constructors(&self) -> TokenStream {
        match self.settings.checking() {
            None => Self::unchecked(&self.ident, &self.generics, &self.field),
            Some(check) => Self::checked(&self.ident, &self.generics, &self.field, check),
        }
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
                    impl #implemented ::newtype::Validated for #name #arguments #bounds {
                        type Inner = #inner;
                        type Error = #error;

                        fn try_new(inner: Self::Inner) -> ::core::result::Result<Self, Self::Error> {
                            #function(&inner)?;
                            ::core::result::Result::Ok(Self { #member: inner })
                        }

                        fn into_inner(self) -> Self::Inner {
                            self.#member
                        }
                    }
        }
    }

    pub fn arbitrary(mut self) -> TokenStream {
        let name = &self.ident;
        let inner = self.field.inner();
        let required = self.settings.required(inner);
        self.generics.make_where_clause().predicates.extend(required);
        let (implemented, arguments, bounds) = self.generics.split_for_impl();
        let mapped = self.settings.checking().map_or_else(Self::plain_strategy, |_| Check::strategy());
        let parameters = self.settings.parameters(inner);
        let source = self.settings.source(inner);
        quote! {
            impl #implemented ::newtype::proptest::arbitrary::Arbitrary for #name #arguments #bounds {
                type Parameters = #parameters;
                type Strategy = ::newtype::proptest::strategy::BoxedStrategy<Self>;

                fn arbitrary_with(parameters: Self::Parameters) -> Self::Strategy {
                    #(#source)*
                    ::newtype::proptest::strategy::Strategy::boxed(#mapped)
                }
            }
        }
    }

    pub fn display(self) -> TokenStream {
        let name = &self.ident;
        let inner = self.field.inner();
        let member = self.field.member();
        let mut generics = self.generics;
        generics.make_where_clause().predicates.push(parse_quote!(#inner: ::core::fmt::Display));
        let (implemented, arguments, bounds) = generics.split_for_impl();
        quote! {
            impl #implemented ::core::fmt::Display for #name #arguments #bounds {
                fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    ::core::fmt::Display::fmt(&self.#member, formatter)
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
            Err(data) => Settings::declared(&attrs)
                .and_then(|_| Err(OnlyField::refused(DeriveInput { attrs, vis, ident, generics, data }))),
        }
    }
}
