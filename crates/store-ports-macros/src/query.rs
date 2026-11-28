mod bound;
mod typed;

use file_ports::Disk;
use port_core::Run;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{LitStr, Token};

use crate::expanded::Expanding;
use bound::Bound;
use query_cache::{Entry, Invoked, Unexpanded};
use typed::{ColumnName, Typed};

#[derive(Debug)]
pub(crate) struct Query {
    invoked: Invoked,
    bounds: Punctuated<Bound, Token![,]>,
}

impl Expanding for Query {
    fn expansion(self) -> syn::Result<TokenStream> {
        let Query { invoked, bounds } = self;
        let (row, source) = (invoked.row(), invoked.source());
        let failed = |unexpanded: Unexpanded| syn::Error::new(source.span(), unexpanded);
        let run = Run::root();
        let entry: Entry<ColumnName> = Entry::found(&run.port(Disk), &source.value()).map_err(failed)?;
        entry.bound(bounds.len()).map_err(failed)?;
        let sql = LitStr::new(&source.value(), source.span());
        let arguments: Vec<TokenStream> = bounds.iter().map(Bound::encoded).collect();
        let values = quote!(::std::vec![#(#arguments),*]);
        let prepared = row.map_or_else(
            || {
                let written = quote!(::store_ports::Prepared::written);
                Ok(quote!(#written(#sql, #values)))
            },
            |columns| {
                let described = entry.columns();
                let columned: syn::Result<Vec<Typed>> =
                    described.iter().map(|column| Typed::described(column, source.span())).collect();
                columned.map(|typed| {
                    let read = quote!(::store_ports::Prepared::read);
                    let taken = quote!(::store_ports::Values);
                    let rows = quote!(::store_ports::Columns);
                    quote! {
                        #read(#sql, #values, |values: &mut #taken| {
                            type __Raw = <#columns as #rows>::Raw;
                            let raw = __Raw { #(#typed)* };
                            <#columns as #rows>::from_raw(raw)
                        })
                    }
                })
            },
        )?;
        Ok(prepared)
    }
}

impl Parse for Query {
    fn parse(input: ParseStream<'_>) -> syn::Result<Query> {
        let invoked = input.parse::<Invoked>()?;
        let bounds = Punctuated::parse_terminated.parse2(invoked.rest().clone())?;
        Ok(Query { invoked, bounds })
    }
}
