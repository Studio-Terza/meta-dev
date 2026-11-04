use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

use super::Wrapped;
use super::check::Check;

pub trait Expansion {
    fn of(wrapped: Wrapped) -> TokenStream;
}

#[derive(Debug)]
pub struct Constructors;

#[derive(Debug)]
pub struct Strategy;

#[derive(Debug)]
pub struct Formatting;

impl Expansion for Constructors {
    fn of(wrapped: Wrapped) -> TokenStream {
        match wrapped.settings.checking() {
            None => Wrapped::unchecked(&wrapped.ident, &wrapped.generics, &wrapped.field),
            Some(check) => Wrapped::checked(&wrapped.ident, &wrapped.generics, &wrapped.field, check),
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
        let mapped = wrapped.settings.checking().map_or_else(Wrapped::plain_strategy, |_| Check::strategy());
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
