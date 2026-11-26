use strum::{IntoStaticStr, VariantArray};
use syntax::Ident;
use syntax::parse::{Parse, ParseStream};

#[derive(IntoStaticStr, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Kind {
    Start,
    #[strum(serialize = "state")]
    Passing,
    Terminal,
}

impl Parse for Kind {
    fn parse(input: ParseStream) -> syntax::Result<Kind> {
        let word: Ident = input.parse()?;
        Self::VARIANTS
            .iter()
            .copied()
            .find(|kind| word == <&str>::from(*kind))
            .ok_or_else(|| syntax::Error::new(word.span(), "expected start, state or terminal"))
    }
}
