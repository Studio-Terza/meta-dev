use proc_macro2::TokenStream;
use syn::LitStr;

use crate::expanded::Expanded;

#[derive(Debug)]
pub struct Migrations;

impl Migrations {
    pub fn expanded(tokens: TokenStream) -> TokenStream {
        syn::parse2::<Option<LitStr>>(tokens).map_or_else(
            |error| error.to_compile_error(),
            |path| Expanded::new(sqlx_macros_core::migrate::expand(path)).tokens(),
        )
    }
}

