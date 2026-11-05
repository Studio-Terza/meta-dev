mod boundary;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Ident};

use boundary::VariantName;

pub struct Named {
    ident: Ident,
    names: Vec<VariantName>,
}

impl Named {
    const ENUMS_ONLY: &'static str = "Named derives for an enum of unit variants";
    const ON_ENUM: &'static str = "name belongs on a variant, not on the enum";
    const NO_VARIANTS: &'static str = "Named needs at least one variant to name";
    const REPEATED: &'static str = "two variants have this name, so FromStr could not tell them apart";

    pub fn expanded(tokens: TokenStream) -> TokenStream {
        syn::parse2(tokens)
            .and_then(Self::unattributed)
            .and_then(Self::table)
            .and_then(Self::distinct)
            .map_or_else(|error| error.to_compile_error(), Self::implemented)
    }

    fn unattributed(input: DeriveInput) -> syn::Result<DeriveInput> {
        input
            .attrs
            .iter()
            .find(|attribute| attribute.path().is_ident("name"))
            .map_or(Ok(()), |attribute| Err(Error::new_spanned(attribute, Self::ON_ENUM)))
            .map(|()| input)
    }

    fn table(DeriveInput { ident, data, .. }: DeriveInput) -> syn::Result<Named> {
        match data {
            Data::Enum(declared) => declared
                .variants
                .into_iter()
                .map(VariantName::try_from)
                .collect::<syn::Result<Vec<VariantName>>>()
                .map(|names| Named { ident, names }),
            Data::Struct(_) | Data::Union(_) => Err(Error::new_spanned(ident, Self::ENUMS_ONLY)),
        }
    }

    fn distinct(self) -> syn::Result<Named> {
        let empty = self.names.is_empty().then(|| Error::new_spanned(&self.ident, Self::NO_VARIANTS));
        let repeated = self
            .names
            .iter()
            .enumerate()
            .find(|(index, name)| self.names.iter().take(*index).any(|earlier| earlier.text() == name.text()))
            .map(|(_, name)| Error::new(name.span(), Self::REPEATED));
        empty.or(repeated).map_or(Ok(self), Err)
    }

    fn implemented(self) -> TokenStream {
        let ident = &self.ident;
        let arms = self.names.iter().map(|name| name.arm(ident));
        quote! {
            impl #ident {
                #[must_use]
                pub const fn name(self) -> &'static str {
                    match self {
                        #(#arms,)*
                    }
                }
            }

            impl ::core::fmt::Display for #ident {
                fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    formatter.pad(Self::name(*self))
                }
            }

            impl ::core::str::FromStr for #ident {
                type Err = ::newtype::Unnamed<#ident>;

                fn from_str(text: &str) -> ::core::result::Result<#ident, ::newtype::Unnamed<#ident>> {
                    <#ident as ::strum::VariantArray>::VARIANTS
                        .iter()
                        .copied()
                        .find(|variant| variant.name() == text)
                        .ok_or_else(|| ::newtype::Unnamed::new(text))
                }
            }
        }
    }
}
