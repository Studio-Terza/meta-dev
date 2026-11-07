mod wrapped;

use proc_macro::TokenStream;
use wrapped::{Constructors, Deserializing, Formatting, Serializing, Strategy, Wrapped};

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

#[proc_macro_derive(Deserialize, attributes(newtype))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    Wrapped::expanded::<Deserializing>(input.into()).into()
}

#[proc_macro_derive(Serialize, attributes(newtype))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    Wrapped::expanded::<Serializing>(input.into()).into()
}
