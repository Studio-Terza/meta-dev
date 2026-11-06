mod boundary;

use proc_macro2::TokenStream;

pub use boundary::Rewritten;

#[derive(Debug)]
pub struct Expanded(sqlx_macros_core::Result<TokenStream>);

impl Expanded {
    pub fn new(expansion: sqlx_macros_core::Result<TokenStream>) -> Expanded {
        Expanded(expansion)
    }

    pub fn tokens(self) -> TokenStream {
        self.0.map_or_else(Rewritten::refused, |tokens| Rewritten::from(tokens).into())
    }
}
