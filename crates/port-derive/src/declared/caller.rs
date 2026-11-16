use syntax::TokenStream;
use syntax::quote;

use super::{Declared, Rendered};

pub struct Caller;

impl Rendered for Caller {
    fn rendered(declared: &Declared) -> TokenStream {
        let item = declared.sealed_trait();
        quote!(#item)
    }
}
