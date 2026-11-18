use syntax::parse::{Parse, ParseStream};
use syntax::{Ident, Token};

pub struct Header;

impl Header {
    const STORED: &'static str = "stored";

    fn stored(input: ParseStream) -> syntax::Result<Header> {
        let word: Ident = input.parse()?;
        Some(&word)
            .filter(|spelled| **spelled == Self::STORED)
            .ok_or_else(|| syntax::Error::new(word.span(), "expected mod or stored"))?;
        input.parse::<Ident>()?;
        input.parse::<Token![in]>()?;
        input.parse::<Ident>().map(|_proofs| Header)
    }
}

impl Parse for Header {
    fn parse(input: ParseStream) -> syntax::Result<Header> {
        match input.parse::<Option<Token![mod]>>()? {
            Some(_module) => input.parse::<Ident>().map(|_lifecycle| Header),
            None => Header::stored(input),
        }
    }
}
