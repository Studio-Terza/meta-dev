#![deny(clippy::wildcard_enum_match_arm)]

mod boundary;

use proc_macro::TokenStream;
use syntax::{DeriveInput, Error};

use boundary::Record;

#[proc_macro_derive(Record, attributes(field, optional, repeated, past, record))]
pub fn derive_record(input: TokenStream) -> TokenStream {
    Derived::expanded(syntax::parse::parse2::<DeriveInput>(input.into()).and_then(Record::try_from)).into()
}

struct Derived;

impl Derived {
    fn expanded(read: Result<Record, Error>) -> syntax::TokenStream {
        match read {
            Ok(record) => syntax::TokenStream::from(&record),
            Err(error) => error.to_compile_error(),
        }
    }
}
