use proc_macro2::TokenStream;
use quote::quote;
use syn::{GenericParam, LifetimeParam, parse_quote};

use super::Wrapped;
use super::settings::Checking;

pub trait Expansion {
    fn of(wrapped: Wrapped) -> TokenStream;
}

#[derive(Debug)]
pub struct Constructors;

#[derive(Debug)]
pub struct Strategy;

#[derive(Debug)]
pub struct Formatting;

#[derive(Debug)]
pub struct Deserializing;

#[derive(Debug)]
pub struct Serializing;

impl Constructors {
    fn chosen(checking: Checking, opened: TokenStream, guarded: TokenStream) -> TokenStream {
        match checking {
            Checking::Unchecked => opened,
            Checking::Checked => guarded,
        }
    }
}

impl Strategy {
    fn mapped(checking: Checking) -> TokenStream {
        match checking {
            Checking::Unchecked => quote! {
                ::newtype::proptest::strategy::Strategy::prop_map(inner, <Self as ::newtype::Newtype>::new)
            },
            Checking::Checked => quote! {
                ::newtype::proptest::strategy::Strategy::prop_filter_map(
                    inner,
                    "the check refuses this value",
                    |value| Self::try_new(value).ok(),
                )
            },
        }
    }
}

impl Deserializing {
    fn built(checking: Checking) -> TokenStream {
        match checking {
            Checking::Unchecked => quote!(::core::result::Result::map(read, Self::new)),
            Checking::Checked => quote! {
                ::core::result::Result::and_then(read, |inner| {
                    ::core::result::Result::map_err(
                        Self::try_new(inner),
                        <Arriving::Error as ::newtype::serde::de::Error>::custom,
                    )
                })
            },
        }
    }
}

impl Expansion for Constructors {
    fn of(wrapped: Wrapped) -> TokenStream {
        let name = &wrapped.ident;
        let (implemented, arguments, bounds) = wrapped.generics.split_for_impl();
        let member = wrapped.field.member();
        let inner = wrapped.field.inner();
        let opened = quote! {
            impl #implemented #name #arguments #bounds {
                #[must_use]
                pub const fn new(inner: #inner) -> Self {
                    Self { #member: inner }
                }
            }

            impl #implemented ::newtype::Newtype for #name #arguments #bounds {
                fn new(inner: Self::Inner) -> Self {
                    Self { #member: inner }
                }
            }
        };
        let guarded = quote! {
            impl #implemented #name #arguments #bounds {
                pub fn try_new(inner: #inner) -> ::core::result::Result<Self, <Self as ::newtype::Checked>::Error> {
                    <Self as ::newtype::Wrapping>::admitted(inner, |inner| Self { #member: inner })
                }

                #[must_use]
                pub fn into_inner(self) -> #inner {
                    self.#member
                }
            }
        };
        let entry = Constructors::chosen(wrapped.settings.checking(), opened, guarded);
        quote! {
            #entry

            impl #implemented ::newtype::Wrapping for #name #arguments #bounds {
                type Inner = #inner;

                fn into_inner(self) -> Self::Inner {
                    self.#member
                }
            }
        }
    }
}

impl Expansion for Strategy {
    fn of(mut wrapped: Wrapped) -> TokenStream {
        let name = &wrapped.ident;
        let inner = wrapped.field.inner();
        let required = wrapped.settings.required(inner);
        wrapped.generics.make_where_clause().predicates.extend(required);
        let (implemented, arguments, bounds) = wrapped.generics.split_for_impl();
        let mapped = Strategy::mapped(wrapped.settings.checking());
        let parameters = wrapped.settings.parameters(inner);
        let source = wrapped.settings.source(inner);
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
}

impl Expansion for Formatting {
    fn of(wrapped: Wrapped) -> TokenStream {
        let name = &wrapped.ident;
        let inner = wrapped.field.inner();
        let member = wrapped.field.member();
        let mut generics = wrapped.generics;
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
}

impl Expansion for Deserializing {
    fn of(wrapped: Wrapped) -> TokenStream {
        let name = &wrapped.ident;
        let inner = wrapped.field.inner();
        let built = Deserializing::built(wrapped.settings.checking());
        let (_, arguments, _) = wrapped.generics.split_for_impl();
        let named = quote!(#name #arguments);
        let mut borrowing = wrapped.generics;
        borrowing.params.insert(0, GenericParam::Lifetime(LifetimeParam::new(parse_quote!('newtype))));
        borrowing.make_where_clause().predicates.push(parse_quote!(#inner: ::newtype::serde::Deserialize<'newtype>));
        let (implemented, _, bounds) = borrowing.split_for_impl();
        quote! {
            impl #implemented ::newtype::serde::Deserialize<'newtype> for #named #bounds {
                fn deserialize<Arriving: ::newtype::serde::Deserializer<'newtype>>(
                    arriving: Arriving,
                ) -> ::core::result::Result<Self, Arriving::Error> {
                    let read = <#inner as ::newtype::serde::Deserialize<'newtype>>::deserialize(arriving);
                    #built
                }
            }
        }
    }
}

impl Expansion for Serializing {
    fn of(wrapped: Wrapped) -> TokenStream {
        let name = &wrapped.ident;
        let inner = wrapped.field.inner();
        let member = wrapped.field.member();
        let mut generics = wrapped.generics;
        generics.make_where_clause().predicates.push(parse_quote!(#inner: ::newtype::serde::Serialize));
        let (implemented, arguments, bounds) = generics.split_for_impl();
        quote! {
            impl #implemented ::newtype::serde::Serialize for #name #arguments #bounds {
                fn serialize<Leaving: ::newtype::serde::Serializer>(
                    &self,
                    leaving: Leaving,
                ) -> ::core::result::Result<Leaving::Ok, Leaving::Error> {
                    ::newtype::serde::Serialize::serialize(&self.#member, leaving)
                }
            }
        }
    }
}
