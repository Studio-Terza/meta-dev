use syn::{Ident, LitStr};

use super::Story;

impl Story {
    pub(super) fn named(ident: &Ident) -> LitStr {
        LitStr::new(&ident.to_string(), ident.span())
    }
}
