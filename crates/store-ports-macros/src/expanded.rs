use proc_macro2::TokenStream;
use syn::parse::Parse;

pub trait Expanding: Parse {
    fn expansion(self) -> syn::Result<TokenStream>;
}

#[derive(Debug)]
pub struct Expanded;

impl Expanded {
    pub fn parsed<Input: Expanding>(tokens: TokenStream) -> TokenStream {
        match syn::parse2::<Input>(tokens).and_then(Input::expansion) {
            Ok(expansion) => expansion,
            Err(error) => error.to_compile_error(),
        }
    }
}
