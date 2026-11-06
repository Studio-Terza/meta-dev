mod declared;

use proc_macro::TokenStream;

use declared::Declared;

#[proc_macro_attribute]
pub fn port(attributes: TokenStream, item: TokenStream) -> TokenStream {
    Declared::expanded(attributes.into(), item.into()).into()
}
