use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Type;

use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Spelling(String);

impl From<TokenStream> for Spelling {
    fn from(tokens: TokenStream) -> Spelling {
        Spelling(tokens.to_string())
    }
}

impl From<&Type> for Spelling {
    fn from(ty: &Type) -> Spelling {
        Spelling(ty.to_token_stream().to_string())
    }
}

impl Spelling {
    const INNER: &'static str = "Inner";

    pub fn inner() -> Spelling {
        Spelling(Self::INNER.to_owned())
    }

    pub fn returning(tokens: Option<TokenStream>) -> Spelling {
        Spelling(tokens.map_or_else(String::new, |ty| format!(" -> {ty}")))
    }

    pub fn signature(generics: &Spelling, inputs: &[Spelling], output: &Spelling) -> Spelling {
        let spelled: Vec<&str> = inputs.iter().map(|input| input.0.as_str()).collect();
        Spelling(format!("{generics}({}){output}", spelled.join(", ")))
    }
}
