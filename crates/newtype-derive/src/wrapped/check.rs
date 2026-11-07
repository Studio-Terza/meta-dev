use proc_macro2::TokenStream;
use quote::quote;
use syn::{Path, Type};

pub struct Check {
    function: Path,
    error: Type,
}

impl Check {
    pub const fn new(function: Path, error: Type) -> Self {
        Self { function, error }
    }

    pub const fn function(&self) -> &Path {
        &self.function
    }

    pub const fn error(&self) -> &Type {
        &self.error
    }

    pub fn strategy() -> TokenStream {
        quote! {
            ::newtype::proptest::strategy::Strategy::prop_filter_map(
                inner,
                "the check refuses this value",
                |value| Self::try_new(value).ok(),
            )
        }
    }
}
