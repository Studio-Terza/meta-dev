use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Generics, Ident};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Spelled,
}

pub struct Marked<'item> {
    ident: &'item Ident,
    generics: &'item Generics,
}

impl<'item> Marked<'item> {
    pub fn new(ident: &'item Ident, generics: &'item Generics) -> Marked<'item> {
        Marked { ident, generics }
    }

    pub fn expanded(tokens: TokenStream, mark: Mark) -> TokenStream {
        let expansion =
            syn::parse2::<DeriveInput>(tokens).map(|input| Marked::new(&input.ident, &input.generics).marked(mark));
        match expansion {
            Ok(expanded) => expanded,
            Err(error) => error.to_compile_error(),
        }
    }

    fn marked(&self, mark: Mark) -> TokenStream {
        match mark {
            Mark::Spelled => self.stored(&quote!(::newtype::Spelled)),
        }
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
