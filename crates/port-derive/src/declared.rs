mod boundary;
mod caller;
mod definition;
mod delegation;
mod filler;
#[cfg(feature = "laws")]
mod laws;
mod method;
mod names;
mod ops;
mod scripted;

use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemTrait;

pub(crate) use boundary::{Constant, Spelling};
pub(crate) use definition::Declared;

use caller::Caller;
use definition::{Rendered, Scripting};
use delegation::Delegation;
use filler::Filler;
use ops::Ops;
use scripted::Scripted;

impl Declared {
    pub(crate) fn expanded(attributes: TokenStream, item: TokenStream) -> TokenStream {
        let expansion = syn::parse2::<Scripting>(attributes).and_then(|scripting| {
            syn::parse2::<ItemTrait>(item)
                .map(|declared| Declared::parsed(declared, scripting))
                .map(|declared| declared.expansion())
        });
        match expansion {
            Ok(expanded) => expanded,
            Err(error) => error.to_compile_error(),
        }
    }

    pub(crate) fn expansion(&self) -> TokenStream {
        let caller = Caller::rendered(self);
        let filler = Filler::rendered_after(self, &caller);
        let ops = Ops::rendered_after(self, &filler);
        let delegated = Delegation::rendered_after(self, &ops);
        let scripted = Scripted::rendered(self);
        quote! {
            #delegated
            #scripted
        }
    }
}
