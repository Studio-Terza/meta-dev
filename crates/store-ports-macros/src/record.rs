mod boundary;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, Ident, LitStr, Type, Visibility};

use crate::expanded::Expanding;
use query_cache::Origin;

#[derive(Debug)]
pub(crate) struct Record {
    visibility: Visibility,
    row: Ident,
    table: LitStr,
    fields: Vec<Placed>,
}

#[derive(Debug)]
struct Placed {
    name: Ident,
    column: Type,
    origin: Origin,
}

impl Placed {
    fn bound(&self) -> TokenStream {
        let Placed { name, column, .. } = self;
        quote!(#name: #column)
    }
}

impl Expanding for Record {
    fn expansion(self) -> syn::Result<TokenStream> {
        let Record { visibility, row, table, fields } = self;
        let key = fields
            .iter()
            .find(|field| Origin::KEYS.contains(&field.origin))
            .ok_or_else(|| Error::new(table.span(), Record::KEYLESS))?;
        let [create, fetch, update, delete, list] = Record::statements(&table, &fields);
        let overwrites = fields.iter().map(|field| field.overwrite(&table));
        let names: Vec<&Ident> = fields.iter().map(|field| &field.name).collect();
        let fresh = format_ident!("{}New", row);
        let given: Vec<&Placed> = fields.iter().filter(|field| Origin::INSERTED.contains(&field.origin)).collect();
        let given_names: Vec<&Ident> = given.iter().map(|field| &field.name).collect();
        let given_types: Vec<&Type> = given.iter().map(|field| &field.column).collect();
        let inserted = given.iter().map(|field| field.bound());
        let revised_fields: Vec<TokenStream> = fields
            .iter()
            .map(|field| {
                let name = &field.name;
                Some(field.origin).filter(|origin| *origin == Origin::Given).map_or_else(
                    || quote!(#name: ::core::clone::Clone::clone(&self.#name)),
                    |_| quote!(#name: ::core::clone::Clone::clone(&new.#name)),
                )
            })
            .collect();
        let assigned = fields.iter().filter(|field| Origin::ASSIGNED.contains(&field.origin)).map(Placed::bound);
        let key_name = &key.name;
        let key_type = &key.column;
        let keyed = key.bound();
        let query = quote!(::store_ports::query!);
        let prepared = quote!(::store_ports::Prepared);
        let reachable = match &visibility {
            Visibility::Inherited => TokenStream::new(),
            Visibility::Public(_) | Visibility::Restricted(_) => quote!(pub),
        };
        Ok(quote! {
            #[derive(Clone, Debug, PartialEq, Eq)]
            #visibility struct #fresh {
                #(#reachable #given_names: #given_types,)*
            }

            impl ::core::convert::From<&#row> for #fresh {
                fn from(record: &#row) -> #fresh {
                    #fresh { #(#given_names: ::core::clone::Clone::clone(&record.#given_names),)* }
                }
            }

            impl ::store_ports::Record for #row {
                type Key = #key_type;

                type New = #fresh;

                fn key(&self) -> &#key_type {
                    &self.#key_name
                }

                fn revised(&self, new: &#fresh) -> #row {
                    #row { #(#revised_fields,)* }
                }

                fn created(new: &#fresh) -> #prepared<Self> {
                    let #fresh { #(#given_names),* } = new;
                    #query(#row, #create, #(#inserted),*)
                }

                fn fetched(key: &#key_type) -> #prepared<Self> {
                    let #key_name = key;
                    #query(#row, #fetch, #keyed)
                }

                fn updated(&self) -> #prepared<()> {
                    let #row { #(#names),* } = self;
                    #query(#update, #(#assigned,)* #keyed)
                }

                fn deleted(key: &#key_type) -> #prepared<()> {
                    let #key_name = key;
                    #query(#delete, #keyed)
                }

                fn listed() -> #prepared<Self> {
                    #query(#row, #list)
                }

                fn overwrites(argument: &::store_ports::Argument) -> ::std::vec::Vec<#prepared<()>> {
                    ::std::vec![#(#prepared::written(#overwrites, ::std::vec![::core::clone::Clone::clone(argument)]),)*]
                }
            }
        })
    }
}
