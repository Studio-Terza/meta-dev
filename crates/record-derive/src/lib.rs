#![deny(clippy::wildcard_enum_match_arm)]

mod boundary;

use proc_macro::TokenStream;
use syntax::{DeriveInput, Error};

use boundary::Record;

#[proc_macro_derive(Record, attributes(field, optional, repeated, past, record))]
pub fn derive_record(input: TokenStream) -> TokenStream {
    let read = syntax::parse::parse2::<DeriveInput>(input.into()).and_then(Record::try_from);
    read.map_or_else(|error: Error| error.to_compile_error(), |record| syntax::TokenStream::from(&record)).into()
}
