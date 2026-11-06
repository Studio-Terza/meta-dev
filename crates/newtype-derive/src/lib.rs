mod marked;
mod wrapped;

use marked::Marked;
use proc_macro::TokenStream;
use wrapped::{Constructors, Formatting, Strategy, Wrapped};

#[proc_macro_derive(Newtype, attributes(newtype))]
pub fn derive_newtype(input: TokenStream) -> TokenStream {
    Wrapped::expanded::<Constructors>(input.into()).into()
}

#[proc_macro_derive(Arbitrary, attributes(newtype))]
pub fn derive_arbitrary(input: TokenStream) -> TokenStream {
    Wrapped::expanded::<Strategy>(input.into()).into()
}

#[proc_macro_derive(Display, attributes(newtype))]
pub fn derive_display(input: TokenStream) -> TokenStream {
    Wrapped::expanded::<Formatting>(input.into()).into()
}

#[proc_macro_derive(Spelled)]
pub fn derive_spelled(input: TokenStream) -> TokenStream {
    Marked::spelled(input.into()).into()
}
