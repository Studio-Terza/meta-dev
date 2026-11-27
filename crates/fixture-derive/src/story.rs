mod boundary;

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(crate) struct Story;

impl Story {
    pub(crate) fn expanded(tokens: TokenStream) -> TokenStream {
        match syn::parse2::<DeriveInput>(tokens) {
            Ok(input) => Story::anchored(&input),
            Err(error) => error.to_compile_error(),
        }
    }

    fn anchored(input: &DeriveInput) -> TokenStream {
        let ident = &input.ident;
        let named = Story::named(ident);
        let (implemented, arguments, bounds) = input.generics.split_for_impl();
        quote! {
            impl #implemented ::fixture::Fixture for #ident #arguments #bounds {
                const STORY: ::fixture::Story =
                    ::fixture::Story::anchored(::core::file!(), ::core::env!("CARGO_MANIFEST_DIR"), #named);
            }
        }
    }
}
