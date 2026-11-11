mod columns;
mod expanded;
mod migrations;
mod query;

use columns::Columns;
use expanded::Expanded;
use migrations::Migrations;
use proc_macro::TokenStream;
use query::Query;

#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    Expanded::parsed::<Query>(input.into()).into()
}

#[proc_macro]
pub fn migrate(input: TokenStream) -> TokenStream {
    Expanded::parsed::<Migrations>(input.into()).into()
}

#[proc_macro_derive(Columns)]
pub fn derive_columns(input: TokenStream) -> TokenStream {
    Expanded::parsed::<Columns>(input.into()).into()
}
