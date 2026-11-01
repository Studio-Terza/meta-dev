use syn::Ident;
use syn::parse::{Parse, ParseStream};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Start,
    Passing,
    Terminal,
}

impl Parse for Kind {
    fn parse(input: ParseStream) -> syn::Result<Kind> {
        let word: Ident = input.parse()?;
        if word == "start" {
            Ok(Kind::Start)
        } else if word == "state" {
            Ok(Kind::Passing)
        } else if word == "terminal" {
            Ok(Kind::Terminal)
        } else {
            Err(syn::Error::new(word.span(), "expected start, state or terminal"))
        }
    }
}
