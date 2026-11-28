use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
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
        let fields = match &input.data {
            Data::Struct(structure) => Ok(&structure.fields),
            Data::Enum(_) | Data::Union(_) => Err(syn::Error::new_spanned(name, Joined::REFUSED)),
        }?;
        let identity = Joined::fieldwise(fields, |_| quote!(::core::default::Default::default()));
        let joined = Joined::fieldwise(fields, |field| quote!(::pipeline::Measure::joined(self.#field, other.#field)));
        Ok(quote! {
            impl #implemented ::core::default::Default for #name #arguments #bounds {
                fn default() -> Self {
                    #identity
                }
            }

            impl #implemented ::pipeline::Measure for #name #arguments #bounds {
                fn joined(self, other: Self) -> Self {
                    #joined
                }
            }
        })
    }

    fn fieldwise(fields: &Fields, each: impl Fn(TokenStream) -> TokenStream) -> TokenStream {
        match fields {
            Fields::Named(listed) => {
                let names = listed.named.iter().filter_map(|field| field.ident.as_ref());
                let values = names.clone().map(|name| each(name.to_token_stream()));
                quote!(Self { #(#names: #values),* })
            }
            Fields::Unnamed(listed) => {
                let values = (0..listed.unnamed.len()).map(|place| each(Index::from(place).to_token_stream()));
                quote!(Self(#(#values),*))
            }
            Fields::Unit => quote!(Self),
        }
    }
}
