use proc_macro2::TokenStream;
use syn::LitStr;
use syn::parse::{Parse, ParseStream};

use crate::expanded::{Expanded, Expanding};

#[derive(Debug)]
pub struct Migrations(Option<LitStr>);

impl Parse for Migrations {
    fn parse(input: ParseStream<'_>) -> syn::Result<Migrations> {
        input.parse().map(Migrations)
    }
}

impl Expanding for Migrations {
    fn expansion(self) -> TokenStream {
        Expanded::from(sqlx_macros_core::migrate::expand(self.0)).into()
    }
}
