use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Generics, Ident};

use super::boundary::{Naming, Spelling};

pub struct Named {
    ident: Ident,
    generics: Generics,
}

impl Named {
    pub fn parse(tokens: TokenStream) -> syn::Result<Named> {
        syn::parse2::<DeriveInput>(tokens).map(|DeriveInput { ident, generics, .. }| Named { ident, generics })
    }

    pub fn op_name(&self, naming: Naming) -> TokenStream {
        let path = Spelling::type_path(&self.ident, naming);
        quote! { ::port_core::OpName::new(concat!(module_path!(), #path)) }
    }

    pub fn implementing(&self, implemented: &TokenStream, items: &TokenStream) -> TokenStream {
        let name = &self.ident;
        let (parameters, arguments, bounds) = self.generics.split_for_impl();
        quote! {
            impl #parameters #implemented for #name #arguments #bounds {
                #items
            }
        }
    }
}
