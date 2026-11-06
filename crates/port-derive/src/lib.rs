mod boundary;
mod declared;
mod named;
mod op;
mod resource;

#[cfg(test)]
mod tests;

use proc_macro::TokenStream;

use declared::Declared;
use op::OpDerive;
use resource::ResourceDerive;

#[proc_macro_attribute]
pub fn port(attributes: TokenStream, item: TokenStream) -> TokenStream {
    Declared::expanded(attributes.into(), item.into()).into()
}

#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    ResourceDerive::expanded(input.into()).into()
}

#[proc_macro_derive(Op)]
pub fn derive_op(input: TokenStream) -> TokenStream {
    OpDerive::expanded(input.into()).into()
}
