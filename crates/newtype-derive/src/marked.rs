use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Generics, Ident, Visibility};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Spelled,
    Listed,
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
        let expansion = syn::parse2::<DeriveInput>(tokens)
            .map(|input| Marked::new(&input.ident, &input.generics).marked(mark, &input.vis));
        match expansion {
            Ok(expanded) => expanded,
            Err(error) => error.to_compile_error(),
        }
    }

    fn marked(&self, mark: Mark, visibility: &Visibility) -> TokenStream {
        match mark {
            Mark::Spelled => self.stored(&quote!(::newtype::Spelled)),
            Mark::Listed => self.listed(visibility),
        }
    }

    fn listed(&self, visibility: &Visibility) -> TokenStream {
        let name = self.ident;
        let unknown = format_ident!("Unknown{}", name);
        let stored = self.stored(&quote!(::newtype::Listed));
        quote! {
            #stored

            #[derive(Debug)]
            #visibility struct #unknown(::std::string::String);

            impl ::core::fmt::Display for #unknown {
                fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    ::core::write!(formatter, "{:?} names no {}", self.0, ::core::stringify!(#name))
                }
            }

            impl ::std::error::Error for #unknown {}

            impl ::core::str::FromStr for #name {
                type Err = #unknown;

                fn from_str(text: &str) -> ::core::result::Result<#name, #unknown> {
                    <#name as ::newtype::Enumerated>::VARIANTS
                        .iter()
                        .copied()
                        .find(|variant| <&'static str as ::core::convert::From<#name>>::from(*variant) == text)
                        .ok_or_else(|| #unknown(::std::borrow::ToOwned::to_owned(text)))
                }
            }
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
