use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Index};

pub struct Joined;

impl Joined {
    const REFUSED: &'static str = "Measure joins a struct fieldwise; an enum or a union has no canonical join";

    pub fn expanded(tokens: TokenStream) -> TokenStream {
        match syn::parse2::<DeriveInput>(tokens).and_then(|input| Joined::implemented(&input)) {
            Ok(expansion) => expansion,
            Err(error) => error.to_compile_error(),
        }
    }

    fn implemented(input: &DeriveInput) -> syn::Result<TokenStream> {
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
