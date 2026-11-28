mod boundary;
mod caller;
mod definition;
mod delegation;
mod filler;
mod method;
mod names;
mod ops;

use proc_macro2::TokenStream;
use syn::ItemTrait;
use syn::parse::Nothing;

pub use boundary::{Constant, Spelling};
pub use definition::Declared;

use caller::Caller;
use definition::Rendered;
use delegation::Delegation;
use filler::Filler;
use ops::Ops;

impl Declared {
    pub fn expanded(attributes: TokenStream, item: TokenStream) -> TokenStream {
        let expansion = syn::parse2::<Nothing>(attributes)
            .and_then(|Nothing| syn::parse2::<ItemTrait>(item))
            .map(Declared::parsed)
            .map(|declared| Self::expansion(&declared));
        match expansion {
            Ok(expanded) => expanded,
            Err(error) => error.to_compile_error(),
        }
    }

    pub fn expansion(&self) -> TokenStream {
        let caller = Caller::rendered(self);
        let filler = Filler::rendered_after(self, &caller);
        let ops = Ops::rendered_after(self, &filler);
        Delegation::rendered_after(self, &ops)
    }
}
