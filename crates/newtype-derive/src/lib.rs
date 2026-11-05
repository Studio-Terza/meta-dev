mod named;
mod variants;
mod wrapped;

use named::Named;
use proc_macro::TokenStream;
use variants::Variants;
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

#[proc_macro_derive(Named, attributes(name, accessor, accessor_value))]
pub fn derive_named(input: TokenStream) -> TokenStream {
    Named::expanded(input.into()).into()
}

#[proc_macro_derive(VariantArray)]
pub fn derive_variant_array(input: TokenStream) -> TokenStream {
    Variants::expanded(input.into()).into()
}
