mod boundary;

use syntax::{Error, Ident, LitStr, TokenStream, Type, Visibility, format_ident, quote};

use crate::expanded::Expanding;
use query_cache::Origin;

#[derive(Debug)]
pub struct Record {
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
    fn expansion(self) -> syntax::Result<TokenStream> {
        let Record { visibility, row, table, fields } = self;
        let key = fields
            .iter()
            .find(|field| Origin::KEYS.contains(&field.origin))
            .ok_or_else(|| Error::new(table.span(), Record::KEYLESS))?;
        let [create, fetch, update, delete] = Record::statements(&table, &fields);
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
        Ok(quote! {
            #[derive(Clone, Debug, PartialEq, Eq)]
            #visibility struct #fresh {
                #(#visibility #given_names: #given_types,)*
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
            }
        })
    }
}
