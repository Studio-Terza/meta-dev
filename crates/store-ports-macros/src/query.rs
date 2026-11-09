mod bound;

use proc_macro2::TokenStream;
use quote::quote;
use sqlx_macros_core::FOSS_DRIVERS;
use sqlx_macros_core::query::{QueryMacroInput, expand_input};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{LitStr, Token, Type};

use crate::expanded::{Expanded, Expanding};
use bound::Bound;

#[derive(Debug)]
pub struct Query {
    row: Option<Type>,
    source: LitStr,
    bounds: Punctuated<Bound, Token![,]>,
}

impl Expanding for Query {
    fn expansion(self) -> TokenStream {
        let Query { row, source, bounds } = self;
        let arguments = bounds.iter().map(Bound::raw);
        let written = quote!(source = #source, args = [#(#arguments),*]);
        row.map_or_else(
            || Self::sqlx(written.clone()),
            |columns| {
                let read = Self::sqlx(quote!(#written, record = __Raw));
                quote! {{
                    type __Raw = <#columns as ::store_ports::Columns>::Raw;
                    (#read).try_map(<#columns as ::store_ports::Columns>::decoded)
                }}
            },
        )
    }
}

impl Query {
    fn sqlx(input: TokenStream) -> TokenStream {
        let expansion = syn::parse2::<QueryMacroInput>(input)
            .map_err(Box::from)
            .and_then(|parsed| expand_input(parsed, FOSS_DRIVERS));
        Expanded::from(expansion).into()
    }

    fn row(input: ParseStream<'_>) -> syn::Result<Option<Type>> {
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
    fn parse(input: ParseStream<'_>) -> syn::Result<Query> {
        let row = Query::row(input)?;
        let source = input.parse()?;
        input.parse::<Option<Token![,]>>()?;
        let bounds = Punctuated::parse_terminated(input)?;
        Ok(Query { row, source, bounds })
    }
}
