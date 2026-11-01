use proc_macro2::TokenStream;
use quote::quote;
use syn::{Path, Type};

pub struct Check {
    function: Path,
    error: Type,
}

impl Check {
    pub fn new(function: Path, error: Type) -> Check {
        Check { function, error }
    }

    pub fn function(&self) -> &Path {
        &self.function
    }

    pub fn error(&self) -> &Type {
        &self.error
    }

    pub fn strategy() -> TokenStream {
        quote! {
            ::proptest::strategy::Strategy::prop_filter_map(
                inner,
                "the check refuses this value",
                |value| <Self as ::newtype::Validated>::try_new(value).ok(),
            )
        }
    }
}
