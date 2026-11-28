use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, Fields, Index};

use crate::{Expansion, Input};

#[derive(Debug)]
pub struct Joined;

impl Expansion for Input<Joined> {
    fn expansion(self) -> syn::Result<TokenStream> {
        let input = self.into_inner();
        let name = &input.ident;
        let (implemented, arguments, bounds) = input.generics.split_for_impl();
        let joined = match &input.data {
            Data::Struct(structure) => Ok(Joined::fieldwise(&structure.fields)),
            Data::Enum(_) | Data::Union(_) => Err(syn::Error::new_spanned(name, Joined::REFUSED)),
        }?;
        Ok(quote! {
            impl #implemented ::pipeline::Measure for #name #arguments #bounds {
                fn joined(self, other: Self) -> Self {
                    #joined
                }
            }
        })
    }
}

impl Joined {
    const REFUSED: &'static str = "Measure joins a struct fieldwise; an enum or a union has no canonical join";

    fn fieldwise(fields: &Fields) -> TokenStream {
        match fields {
            Fields::Named(listed) => {
                let names: Vec<&Ident> = listed.named.iter().filter_map(|field| field.ident.as_ref()).collect();
                quote!(Self { #(#names: ::pipeline::Measure::joined(self.#names, other.#names)),* })
            }
            Fields::Unnamed(listed) => {
                let places: Vec<Index> = (0..listed.unnamed.len()).map(Index::from).collect();
                quote!(Self(#(::pipeline::Measure::joined(self.#places, other.#places)),*))
            }
            Fields::Unit => quote!(Self),
        }
    }
}
