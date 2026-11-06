mod row;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{Data, DataEnum, DeriveInput, Ident, LitStr, Variant, Visibility};

use row::Row;

pub struct Table {
    visibility: Visibility,
    ident: Ident,
    rows: Vec<Row>,
}

impl Table {
    pub fn expanded(tokens: TokenStream) -> TokenStream {
        syn::parse2(tokens)
            .and_then(Self::read)
            .map_or_else(|error| error.to_compile_error(), |table| table.generated())
    }

    fn read(DeriveInput { vis, ident, data, .. }: DeriveInput) -> syn::Result<Table> {
        let rows = Self::variants(data).iter().map(Row::declared).collect::<syn::Result<Vec<Vec<Row>>>>()?;
        Ok(Table { visibility: vis, ident, rows: rows.into_iter().flatten().collect() })
    }

    fn variants(data: Data) -> Vec<Variant> {
        match data {
            Data::Enum(DataEnum { variants, .. }) => variants.into_iter().collect(),
            Data::Struct(_) | Data::Union(_) => Vec::new(),
        }
    }

    fn generated(&self) -> TokenStream {
        let Table { visibility, ident, rows } = self;
        let unknown = format_ident!("Unknown{}", ident);
        let variants: Vec<TokenStream> =
            rows.iter().map(Row::variant).map(|variant| quote_spanned!(variant.span()=> Self::#variant)).collect();
        let names: Vec<&LitStr> = rows.iter().map(Row::name).collect();
        quote! {
            #[deny(unreachable_patterns)]
            impl #ident {
                #[must_use]
                #visibility const fn name(self) -> &'static str {
                    match self {
                        #(#variants => #names,)*
                    }
                }
            }

            impl ::newtype::Stored for #ident {
                type Kind = ::newtype::Spelled;
            }

            #[derive(Debug, ::thiserror::Error)]
            #[error("{0:?} names no {kind}", kind = ::core::stringify!(#ident))]
            #visibility struct #unknown(::std::string::String);

            impl ::core::fmt::Display for #ident {
                fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    formatter.write_str(Self::name(*self))
                }
            }

            #[deny(unreachable_patterns)]
            impl ::core::str::FromStr for #ident {
                type Err = #unknown;

                fn from_str(text: &str) -> ::core::result::Result<Self, #unknown> {
                    match text {
                        #(#names => ::core::result::Result::Ok(#variants),)*
                        _ => ::core::result::Result::Err(#unknown(::std::borrow::ToOwned::to_owned(text))),
                    }
                }
            }
        }
    }
}
