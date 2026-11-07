use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Generics, Ident};

pub struct Marked<'item> {
    ident: &'item Ident,
    generics: &'item Generics,
}

impl<'item> Marked<'item> {
    pub fn new(ident: &'item Ident, generics: &'item Generics) -> Marked<'item> {
        Marked { ident, generics }
    }

    pub fn spelled(tokens: TokenStream) -> TokenStream {
        syn::parse2::<DeriveInput>(tokens).map_or_else(
            |error| error.to_compile_error(),
            |input| Marked::new(&input.ident, &input.generics).stored(&quote!(::newtype::Spelled)),
        )
    }

    pub fn stored(&self, kind: &TokenStream) -> TokenStream {
        let name = self.ident;
        let (implemented, arguments, bounds) = self.generics.split_for_impl();
        quote! {
            impl #implemented ::newtype::Stored for #name #arguments #bounds {
                type Kind = #kind;
            }
        }
    }
}
