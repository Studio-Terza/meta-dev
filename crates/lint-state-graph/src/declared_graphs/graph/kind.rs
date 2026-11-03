use syn::Ident;
use syn::parse::{Parse, ParseStream};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Start,
    Passing,
    Terminal,
}

impl Kind {
    const SPELLED: [(&'static str, Kind); 3] =
        [("start", Kind::Start), ("state", Kind::Passing), ("terminal", Kind::Terminal)];
}

impl Parse for Kind {
    fn parse(input: ParseStream) -> syn::Result<Kind> {
        let word: Ident = input.parse()?;
        Self::SPELLED
            .into_iter()
            .find(|(spelled, _)| word == *spelled)
            .map(|(_, kind)| kind)
            .ok_or_else(|| syn::Error::new(word.span(), "expected start, state or terminal"))
    }
}
