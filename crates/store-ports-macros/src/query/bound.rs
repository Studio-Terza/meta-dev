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
    pub fn encoded(&self) -> TokenStream {
        let Bound { name, column } = self;
        let encode = quote!(::ports::store::IntoArgument::into_argument);
        let raw = quote!(::ports::store::Column);
        quote!(#encode(<#column as #raw>::raw(&#name)))
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
