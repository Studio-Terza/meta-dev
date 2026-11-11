mod declared;
mod named;

use proc_macro::TokenStream;

use declared::Declared;
use named::{Derived, Named};

#[proc_macro_attribute]
pub fn port(attributes: TokenStream, item: TokenStream) -> TokenStream {
    Declared::expanded(attributes.into(), item.into()).into()
}

#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    Named::expanded(input.into(), Derived::Resource).into()
}

#[proc_macro_derive(Op)]
pub fn derive_op(input: TokenStream) -> TokenStream {
    Named::expanded(input.into(), Derived::Op).into()
}
