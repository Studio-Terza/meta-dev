use proc_macro2::TokenStream;
use quote::quote;

use super::definition::Declared;

pub struct Caller;

impl Caller {
    pub fn rendered(declared: &Declared) -> TokenStream {
        let item = declared.sealed_trait();
        quote!(#item)
    }
}
