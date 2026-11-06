use proc_macro2::TokenStream;
use quote::quote;

use super::{Declared, Rendered};

pub struct Caller;

impl Rendered for Caller {
    fn rendered(declared: &Declared) -> TokenStream {
        let item = declared.sealed_trait();
        quote!(#item)
    }
}
