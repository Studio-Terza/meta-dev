use proc_macro2::TokenStream;
use syn::parse::Parse;

pub(crate) trait Expanding: Parse {
    fn expansion(self) -> syn::Result<TokenStream>;
}

#[derive(Debug)]
pub(crate) struct Expanded;

impl Expanded {
    pub(crate) fn parsed<Input: Expanding>(tokens: TokenStream) -> TokenStream {
        match syn::parse2::<Input>(tokens).and_then(Input::expansion) {
            Ok(expansion) => expansion,
            Err(error) => error.to_compile_error(),
        }
    }
}
