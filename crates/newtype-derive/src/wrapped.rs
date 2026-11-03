mod check;
mod only_field;
mod settings;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, WherePredicate, parse_quote};

use check::Check;
use only_field::OnlyField;
use settings::Settings;

pub struct Wrapped {
    input: DeriveInput,
    field: OnlyField,
    settings: Settings,
}

impl Wrapped {
    pub fn expanded(tokens: TokenStream, expansion: fn(Wrapped) -> TokenStream) -> TokenStream {
        Self::parsed(tokens).map_or_else(|error| error.to_compile_error(), expansion)
    }

    pub fn constructors(&self) -> TokenStream {
        let name = &self.input.ident;
        let (implemented, arguments, bounds) = self.input.generics.split_for_impl();
        let member = self.field.member();
        let inner = self.field.inner();
        match self.settings.checking() {
            None => {
                let zero = self.field.counts().then(|| {
                    quote! {
                        #[must_use]
                        pub const fn zero() -> Self {
                            Self { #member: 0 }
                        }
                    }
                });
                quote! {
                    impl #implemented #name #arguments #bounds {
                        #[must_use]
                        pub const fn new(inner: #inner) -> Self {
                            Self { #member: inner }
                        }

                        #zero
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
            Some(check) => {
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
        }
    }

    pub fn arbitrary(mut self) -> TokenStream {
        let name = &self.input.ident;
        let inner = self.field.inner().clone();
        let chosen = self.settings.strategy().cloned();
        let required: Vec<WherePredicate> = match chosen {
            None => vec![
                parse_quote!(Self: ::core::fmt::Debug),
                parse_quote!(#inner: ::proptest::arbitrary::Arbitrary + 'static),
                parse_quote!(<#inner as ::proptest::arbitrary::Arbitrary>::Strategy: 'static),
            ],
            Some(_) => vec![parse_quote!(Self: ::core::fmt::Debug), parse_quote!(Self: 'static)],
        };
        self.input.generics.make_where_clause().predicates.extend(required);
        let (implemented, arguments, bounds) = self.input.generics.split_for_impl();
        let mapped = self.settings.checking().map_or_else(Self::plain_strategy, |_| Check::strategy());
        let (parameters, source) = match chosen {
            Some(strategy) => (quote!(()), quote!(let _ = parameters; let inner = #strategy;)),
            None => (
                quote!(<#inner as ::proptest::arbitrary::Arbitrary>::Parameters),
                quote!(let inner = ::proptest::arbitrary::any_with::<#inner>(parameters);),
            ),
        };
        quote! {
            impl #implemented ::proptest::arbitrary::Arbitrary for #name #arguments #bounds {
                type Parameters = #parameters;
                type Strategy = ::proptest::strategy::BoxedStrategy<Self>;

                fn arbitrary_with(parameters: Self::Parameters) -> Self::Strategy {
                    #source
                    ::proptest::strategy::Strategy::boxed(#mapped)
                }
            }
        }
    }

    pub fn display(self) -> TokenStream {
        let name = &self.input.ident;
        let inner = self.field.inner();
        let member = self.field.member();
        let mut generics = self.input.generics.clone();
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
            ::proptest::strategy::Strategy::prop_map(inner, <Self as ::newtype::Newtype>::new)
        }
    }

    fn parsed(tokens: TokenStream) -> syn::Result<Wrapped> {
        let input: DeriveInput = syn::parse2(tokens)?;
        let settings = Settings::declared(&input.attrs)?;
        let field = OnlyField::found_in(&input)?;
        Ok(Wrapped { input, field, settings })
    }
}
