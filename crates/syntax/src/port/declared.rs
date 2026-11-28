mod boundary;
mod caller;
mod definition;
mod delegation;
mod filler;
mod method;
mod names;
mod ops;

use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemTrait;
use syn::parse::{Parse, ParseStream};

pub use boundary::{Constant, Spelling};
pub use definition::Declared;

use crate::Expansion;
use caller::Caller;
use delegation::Delegation;
use filler::Filler;
use ops::Ops;

impl Parse for Declared {
    fn parse(input: ParseStream<'_>) -> syn::Result<Declared> {
        input.parse::<ItemTrait>().map(Declared::parsed)
    }
}

impl Expansion for Declared {
    fn expansion(self) -> syn::Result<TokenStream> {
        let caller = Caller::rendered(&self);
        let filler = Filler::rendered(&self);
        let ops = Ops::rendered(&self);
        let delegation = Delegation::rendered(&self);
        Ok(quote! {
            #caller
            #filler
            #ops
            #delegation
        })
    }
}
