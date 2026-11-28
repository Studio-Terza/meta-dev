use proc_macro2::TokenStream;

#[cfg(feature = "laws")]
use super::definition::Rendered;
use super::definition::{Declared, Scripting};
#[cfg(feature = "laws")]
use super::laws::Laws;

pub(crate) struct Scripted;

impl Scripted {
    pub(crate) fn named(declared: &Declared) -> TokenStream {
        match declared.scripting() {
            Scripting::Unscripted => TokenStream::new(),
            #[cfg(feature = "laws")]
            Scripting::Scripted => Laws::named(declared),
            #[cfg(not(feature = "laws"))]
            Scripting::Scripted => TokenStream::new(),
        }
    }

    pub(crate) fn rendered(declared: &Declared) -> TokenStream {
        match declared.scripting() {
            Scripting::Unscripted => TokenStream::new(),
            #[cfg(feature = "laws")]
            Scripting::Scripted => Laws::rendered(declared),
            #[cfg(not(feature = "laws"))]
            Scripting::Scripted => TokenStream::new(),
        }
    }
}
