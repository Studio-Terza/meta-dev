use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed};

#[derive(Debug)]
pub struct Columns(DeriveInput);

impl Columns {
    const UNNAMED: &'static str = "a row of columns is a struct with named fields";

    pub fn expanded(tokens: TokenStream) -> TokenStream {
        syn::parse2::<DeriveInput>(tokens)
            .and_then(Columns::named)
            .map_or_else(|error| error.to_compile_error(), |(row, fields)| row.generated(&fields))
    }

    fn named(input: DeriveInput) -> syn::Result<(Columns, Vec<Field>)> {
        let fields = Self::fields(&input.data).ok_or_else(|| Error::new_spanned(&input, Self::UNNAMED))?;
        Ok((Columns(input), fields))
    }

    fn fields(data: &Data) -> Option<Vec<Field>> {
        match data {
            Data::Struct(DataStruct { fields: Fields::Named(FieldsNamed { named, .. }), .. }) => {
                Some(named.iter().cloned().collect())
            }
            Data::Struct(_) | Data::Enum(_) | Data::Union(_) => None,
        }
    }

    fn generated(&self, fields: &[Field]) -> TokenStream {
        let DeriveInput { vis, ident, .. } = &self.0;
        let raw = format_ident!("{}Raw", ident);
        let visible = fields.iter().map(|field| &field.vis);
        let names: Vec<_> = fields.iter().map(|field| &field.ident).collect();
        let columns: Vec<_> = fields.iter().map(|field| &field.ty).collect();
        quote! {
            #[doc(hidden)]
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
        }
    }
}
