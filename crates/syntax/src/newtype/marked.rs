use proc_macro2::TokenStream;
use quote::quote;
use syn::{Generics, Ident};

use crate::{Expansion, Input};

#[derive(Debug)]
pub struct Spelled;

pub struct Marked<'item> {
    ident: &'item Ident,
    generics: &'item Generics,
}

impl Expansion for Input<Spelled> {
    fn expansion(self) -> syn::Result<TokenStream> {
        let input = self.into_inner();
        Ok(Marked::new(&input.ident, &input.generics).stored(&quote!(::newtype::Spelled)))
    }
}

impl<'item> Marked<'item> {
    pub fn new(ident: &'item Ident, generics: &'item Generics) -> Marked<'item> {
        Marked { ident, generics }
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
