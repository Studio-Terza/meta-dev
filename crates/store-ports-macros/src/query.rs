mod bound;
mod typed;

use syntax::Punctuated;
use syntax::TokenStream;
use syntax::parse::{Parse, ParseStream};
use syntax::quote;
use syntax::{LitStr, Token, Type};

use crate::expanded::Expanding;
use bound::Bound;
use query_cache::{Entry, Unexpanded};
use typed::{ColumnName, Typed};

#[derive(Debug)]
pub struct Query {
    row: Option<Type>,
    source: LitStr,
    bounds: Punctuated<Bound, Token![,]>,
}

impl Expanding for Query {
    fn expansion(self) -> syntax::Result<TokenStream> {
        let Query { row, source, bounds } = self;
        let failed = |unexpanded: Unexpanded| syntax::Error::new(source.span(), unexpanded);
        let entry: Entry<ColumnName> = Entry::found(&source.value()).map_err(failed)?;
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
                let columned: syntax::Result<Vec<Typed>> =
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

impl Query {
    fn row(input: ParseStream<'_>) -> syntax::Result<Option<Type>> {
        input.fork().parse::<LitStr>().map_or_else(
            |_| {
                let row = input.parse::<Type>()?;
                input.parse::<Token![,]>()?;
                Ok(Some(row))
            },
            |_| Ok(None),
        )
    }
}

impl Parse for Query {
    fn parse(input: ParseStream<'_>) -> syntax::Result<Query> {
        let row = Query::row(input)?;
        let source = input.parse()?;
        input.parse::<Option<Token![,]>>()?;
        let bounds = Punctuated::parse_terminated(input)?;
        Ok(Query { row, source, bounds })
    }
}
