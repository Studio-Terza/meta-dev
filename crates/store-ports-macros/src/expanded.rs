use syntax::TokenStream;
use syntax::parse::Parse;

pub trait Expanding: Parse {
    fn expansion(self) -> syntax::Result<TokenStream>;
}

#[derive(Debug)]
pub struct Expanded;

impl Expanded {
    pub fn parsed<Input: Expanding>(tokens: TokenStream) -> TokenStream {
        syntax::parse::parse2::<Input>(tokens)
            .and_then(Input::expansion)
            .unwrap_or_else(|error| error.to_compile_error())
    }
}
