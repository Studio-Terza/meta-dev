mod boundary;

use proc_macro2::TokenStream;
use syn::parse::Parse;

use boundary::Rewritten;

pub trait Expanding: Parse {
    fn expansion(self) -> TokenStream;
}

#[derive(Debug)]
pub struct Expanded(sqlx_macros_core::Result<TokenStream>);

impl Expanded {
    pub fn parsed<Input: Expanding>(tokens: TokenStream) -> TokenStream {
        syn::parse2::<Input>(tokens).map_or_else(|error| error.to_compile_error(), Input::expansion)
    }
}

impl From<sqlx_macros_core::Result<TokenStream>> for Expanded {
    fn from(expansion: sqlx_macros_core::Result<TokenStream>) -> Expanded {
        Expanded(expansion)
    }
}

impl From<Expanded> for TokenStream {
    fn from(expanded: Expanded) -> TokenStream {
        expanded.0.map_or_else(Rewritten::refused, |tokens| Rewritten::from(tokens).into())
    }
}
