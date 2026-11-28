use std::marker::PhantomData;

use proc_macro2::TokenStream;
use syn::DeriveInput;
use syn::parse::{Nothing, Parse, ParseStream};

pub trait Expansion: Parse {
    fn expansion(self) -> syn::Result<TokenStream>;

    #[must_use]
    fn derived(tokens: TokenStream) -> TokenStream {
        Self::compiled(syn::parse2::<Self>(tokens))
    }

    #[must_use]
    fn attributed(attributes: TokenStream, item: TokenStream) -> TokenStream {
        Self::compiled(syn::parse2::<Nothing>(attributes).and_then(|Nothing| syn::parse2::<Self>(item)))
    }

    #[must_use]
    fn compiled(parsed: syn::Result<Self>) -> TokenStream {
        match parsed.and_then(Self::expansion) {
            Ok(expansion) => expansion,
            Err(error) => error.to_compile_error(),
        }
    }
}

#[derive(Debug)]
pub struct Input<Kind>(DeriveInput, PhantomData<Kind>);

impl<Kind> Input<Kind> {
    pub(crate) fn into_inner(self) -> DeriveInput {
        self.0
    }
}

impl<Kind> From<DeriveInput> for Input<Kind> {
    fn from(input: DeriveInput) -> Input<Kind> {
        Input(input, PhantomData)
    }
}

impl<Kind> Parse for Input<Kind> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Input<Kind>> {
        input.parse::<DeriveInput>().map(Input::from)
    }
}
