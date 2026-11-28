use proc_macro::TokenStream;
use syntax::newtype::{
    CheckedDeserializing, Constructors, Deserializing, Formatting, Generating, Joined, Serializing, Spelled,
};
use syntax::port::{Declared, Op, Resource};
use syntax::store::{Columns, Query};
use syntax::{Expansion, Input};

#[proc_macro_attribute]
pub fn port(attributes: TokenStream, item: TokenStream) -> TokenStream {
    Declared::attributed(attributes.into(), item.into()).into()
}

#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    Input::<Resource>::derived(input.into()).into()
}

#[proc_macro_derive(Op)]
pub fn derive_op(input: TokenStream) -> TokenStream {
    Input::<Op>::derived(input.into()).into()
}

#[proc_macro_derive(Newtype, attributes(newtype))]
pub fn derive_newtype(input: TokenStream) -> TokenStream {
    Input::<Constructors>::derived(input.into()).into()
}

#[proc_macro_derive(Arbitrary, attributes(newtype))]
pub fn derive_arbitrary(input: TokenStream) -> TokenStream {
    Input::<Generating>::derived(input.into()).into()
}

#[proc_macro_derive(Display, attributes(newtype))]
pub fn derive_display(input: TokenStream) -> TokenStream {
    Input::<Formatting>::derived(input.into()).into()
}

#[proc_macro_derive(Deserialize, attributes(newtype))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    Input::<Deserializing>::derived(input.into()).into()
}

#[proc_macro_derive(CheckedDeserialize, attributes(newtype))]
pub fn derive_checked_deserialize(input: TokenStream) -> TokenStream {
    Input::<CheckedDeserializing>::derived(input.into()).into()
}

#[proc_macro_derive(Serialize, attributes(newtype))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    Input::<Serializing>::derived(input.into()).into()
}

#[proc_macro_derive(Measure)]
pub fn derive_measure(input: TokenStream) -> TokenStream {
    Input::<Joined>::derived(input.into()).into()
}

#[proc_macro_derive(Spelled)]
pub fn derive_spelled(input: TokenStream) -> TokenStream {
    Input::<Spelled>::derived(input.into()).into()
}

#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    Query::derived(input.into()).into()
}

#[proc_macro_derive(Columns)]
pub fn derive_columns(input: TokenStream) -> TokenStream {
    Columns::derived(input.into()).into()
}
