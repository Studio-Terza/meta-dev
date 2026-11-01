use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Path, Token, Type};

mod keyword {
    syn::custom_keyword!(check);
    syn::custom_keyword!(error);
}

pub struct Check {
    function: Path,
    error: Type,
}

impl Check {
    const ATTRIBUTE: &str = "newtype";

    pub fn declared(attributes: &[Attribute]) -> syn::Result<Option<Check>> {
        attributes
            .iter()
            .find(|attribute| attribute.path().is_ident(Self::ATTRIBUTE))
            .map(Attribute::parse_args)
            .transpose()
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

impl Parse for Check {
    fn parse(input: ParseStream<'_>) -> syn::Result<Check> {
        input.parse::<keyword::check>()?;
        input.parse::<Token![=]>()?;
        let function = input.parse()?;
        input.parse::<Token![,]>()?;
        input.parse::<keyword::error>()?;
        input.parse::<Token![=]>()?;
        Ok(Check { function, error: input.parse()? })
    }
}
