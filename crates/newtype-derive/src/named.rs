mod boundary;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Ident};

use boundary::{Accessor, VariantName};

pub struct Named {
    ident: Ident,
    names: Vec<VariantName>,
    accessors: Vec<Accessor>,
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
            .and_then(|named| named.names.iter().try_for_each(|name| name.supplies(&named.accessors)).map(|()| named))
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

    fn table(DeriveInput { ident, data, attrs, .. }: DeriveInput) -> syn::Result<Named> {
        match data {
            Data::Enum(declared) => {
                Ok(Named { names: VariantName::all(declared.variants)?, accessors: Accessor::declared(&attrs)?, ident })
            }
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
        let arms = self.names.iter().map(|name| name.arm(ident)).collect::<Vec<_>>();
        let parses = self.names.iter().map(|name| name.parse_arm(ident)).collect::<Vec<_>>();
        let ordinals =
            self.names.iter().enumerate().map(|(index, name)| name.ordinal(ident, index)).collect::<Vec<_>>();
        let accessors = self.accessors.iter().map(|accessor| accessor.method(ident, &self.names));
        let count = self.names.len();
        quote! {
            const _: () = {
                #[diagnostic::on_unimplemented(
                    message = "Named needs an enum that is Copy: add Copy to its derive list",
                    label = "this enum is not Copy"
                )]
                trait NamedNeedsCopy {}

                impl<Of: ::core::marker::Copy> NamedNeedsCopy for Of {}

                const fn copied<Of: NamedNeedsCopy>() {}

                copied::<#ident>();
            };

            const _: () = {
                const fn ordinal(of: &#ident) -> usize {
                    match *of {
                        #(#ordinals,)*
                    }
                }

                let listed = <#ident as ::strum::VariantArray>::VARIANTS;
                let mut wanted = 0;
                while wanted < #count {
                    let mut found = false;
                    let mut at = 0;
                    while at < listed.len() {
                        found |= ordinal(&listed[at]) == wanted;
                        at += 1;
                    }
                    ::core::assert!(found, "Named needs VariantArray to list every variant of the enum exactly once");
                    wanted += 1;
                }
                ::core::assert!(
                    listed.len() == #count,
                    "Named needs VariantArray to list every variant of the enum exactly once"
                );
            };

            impl #ident {
                #[must_use]
                pub const fn name(self) -> &'static str {
                    match self {
                        #(#arms,)*
                    }
                }

                #(#accessors)*
            }

            impl ::core::fmt::Display for #ident {
                fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    formatter.pad(match *self {
                        #(#arms,)*
                    })
                }
            }

            impl ::core::str::FromStr for #ident {
                type Err = ::newtype::Unnamed<#ident>;

                fn from_str(text: &str) -> ::core::result::Result<#ident, ::newtype::Unnamed<#ident>> {
                    match text {
                        #(#parses,)*
                        _ => ::core::result::Result::Err(::newtype::Unnamed::new(text)),
                    }
                }
            }
        }
    }
}
