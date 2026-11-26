use proc_macro2::TokenStream;

pub trait Compiled: Into<syn::Result<TokenStream>> {
    fn compiled(self) -> TokenStream {
        match self.into() {
            Ok(expansion) => expansion,
            Err(error) => error.to_compile_error(),
        }
    }
}

impl Compiled for syn::Result<TokenStream> {}
