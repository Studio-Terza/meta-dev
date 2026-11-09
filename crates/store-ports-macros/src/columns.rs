use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Data, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed};

use crate::expanded::Expanding;

#[derive(Debug)]
pub struct Columns {
    row: DeriveInput,
    fields: Vec<Field>,
}

impl Columns {
    const UNNAMED: &'static str = "a row of columns is a struct with named fields";

    fn fields(data: &Data) -> Option<Vec<Field>> {
        match data {
            Data::Struct(DataStruct { fields: Fields::Named(FieldsNamed { named, .. }), .. }) => {
                Some(named.iter().cloned().collect())
            }
            Data::Struct(_) | Data::Enum(_) | Data::Union(_) => None,
        }
    }
}

impl Parse for Columns {
    fn parse(input: ParseStream<'_>) -> syn::Result<Columns> {
        let row: DeriveInput = input.parse()?;
        let fields = Columns::fields(&row.data).ok_or_else(|| Error::new_spanned(&row, Columns::UNNAMED))?;
        Ok(Columns { row, fields })
    }
}

impl Expanding for Columns {
    fn expansion(self) -> syn::Result<TokenStream> {
        let Columns { row: DeriveInput { vis, ident, .. }, fields } = self;
        let raw = format_ident!("{}Raw", ident);
        let visible = fields.iter().map(|field| &field.vis);
        let names: Vec<_> = fields.iter().map(|field| &field.ident).collect();
        let columns: Vec<_> = fields.iter().map(|field| &field.ty).collect();
        Ok(quote! {
            #[derive(Debug)]
            #vis struct #raw {
                #(#visible #names: <#columns as ::store_ports::Column>::Raw,)*
            }

            impl ::store_ports::Columns for #ident {
                type Raw = #raw;

                fn from_raw(raw: #raw) -> ::core::result::Result<#ident, ::store_ports::ColumnFailure> {
                    ::core::result::Result::Ok(#ident {
                        #(#names: <#columns as ::store_ports::Column>::from_raw(raw.#names)?,)*
                    })
                }
            }
        })
    }
}
