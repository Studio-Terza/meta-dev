use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Attribute, Error, Fields, Ident, LitStr, Variant};

pub struct VariantName {
    variant: Ident,
    text: LitStr,
}

impl VariantName {
    const ATTRIBUTE: &'static str = "name";
    const UNIT_ONLY: &'static str = "a Named variant carries no fields";
    const UNNAMED: &'static str = "a Named variant needs #[name(\"..\")]";

    pub fn text(&self) -> String {
        self.text.value()
    }

    pub fn span(&self) -> Span {
        self.text.span()
    }

    pub fn arm(&self, owner: &Ident) -> TokenStream {
        let variant = &self.variant;
        let text = &self.text;
        quote!(#owner::#variant => #text)
    }

    fn named(variant: Variant) -> syn::Result<VariantName> {
        let text = variant
            .attrs
            .iter()
            .find(|attribute| attribute.path().is_ident(Self::ATTRIBUTE))
            .ok_or_else(|| Error::new_spanned(&variant.ident, Self::UNNAMED))
            .and_then(Attribute::parse_args::<LitStr>)?;
        Ok(VariantName { variant: variant.ident, text })
    }
}

impl TryFrom<Variant> for VariantName {
    type Error = Error;

    fn try_from(variant: Variant) -> syn::Result<VariantName> {
        match variant.fields {
            Fields::Unit => Self::named(variant),
            Fields::Named(_) | Fields::Unnamed(_) => Err(Error::new_spanned(variant, Self::UNIT_ONLY)),
        }
    }
}
