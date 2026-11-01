mod wrapped;

use proc_macro::TokenStream;
use wrapped::Wrapped;

#[proc_macro_derive(Newtype, attributes(newtype))]
pub fn derive_newtype(input: TokenStream) -> TokenStream {
    Wrapped::expanded(input.into(), |wrapped| wrapped.constructors()).into()
}

#[proc_macro_derive(Arbitrary, attributes(newtype))]
pub fn derive_arbitrary(input: TokenStream) -> TokenStream {
    Wrapped::expanded(input.into(), Wrapped::arbitrary).into()
}

#[proc_macro_derive(Display, attributes(newtype))]
pub fn derive_display(input: TokenStream) -> TokenStream {
    Wrapped::expanded(input.into(), Wrapped::display).into()
}
