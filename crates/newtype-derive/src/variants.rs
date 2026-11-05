use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields};

pub struct Variants;

impl Variants {
    const UNIT_ENUMS_ONLY: &'static str = "VariantArray derives for an enum of unit variants";

    pub fn expanded(tokens: TokenStream) -> TokenStream {
        syn::parse2(tokens).and_then(Self::listed).unwrap_or_else(|error| error.to_compile_error())
    }

    fn listed(DeriveInput { ident, data, .. }: DeriveInput) -> syn::Result<TokenStream> {
        match data {
            Data::Enum(declared) if declared.variants.iter().all(|variant| matches!(variant.fields, Fields::Unit)) => {
                let variants = declared.variants.iter().map(|variant| &variant.ident);
                Ok(quote! {
                    impl ::newtype::VariantArray for #ident {
                        const VARIANTS: &'static [#ident] = &[#(#ident::#variants),*];
                    }
                })
            }
            Data::Enum(_) | Data::Struct(_) | Data::Union(_) => Err(Error::new_spanned(ident, Self::UNIT_ENUMS_ONLY)),
        }
    }
}
