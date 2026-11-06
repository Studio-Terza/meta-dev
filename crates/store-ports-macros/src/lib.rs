mod columns;
mod expanded;
mod migrations;
mod query;

use columns::Columns;
use migrations::Migrations;
use proc_macro::TokenStream;
use query::Query;

#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    Query::expanded(input.into()).into()
}

#[proc_macro]
pub fn migrate(input: TokenStream) -> TokenStream {
    Migrations::expanded(input.into()).into()
}

#[proc_macro_derive(Columns)]
pub fn derive_columns(input: TokenStream) -> TokenStream {
    Columns::expanded(input.into()).into()
}
