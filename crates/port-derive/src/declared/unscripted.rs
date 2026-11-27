use proc_macro2::TokenStream;

use super::{Declared, Rendered};

pub(crate) struct Laws;

impl Rendered for Laws {
    fn rendered(_declared: &Declared) -> TokenStream {
        TokenStream::new()
    }
}

impl Laws {
    pub(crate) fn named(_declared: &Declared) -> TokenStream {
        TokenStream::new()
    }
}
