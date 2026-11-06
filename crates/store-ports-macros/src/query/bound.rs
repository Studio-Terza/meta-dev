use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token, Type};

#[derive(Debug)]
pub struct Bound {
    name: Ident,
    column: Type,
}

impl Bound {
    pub fn raw(&self) -> TokenStream {
        let Bound { name, column } = self;
        quote!(<#column as ::store_ports::Column>::into_raw(#name))
    }
}

impl Parse for Bound {
    fn parse(input: ParseStream<'_>) -> syn::Result<Bound> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let column = input.parse()?;
        Ok(Bound { name, column })
    }
}
