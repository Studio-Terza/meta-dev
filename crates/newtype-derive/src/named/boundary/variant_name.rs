use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Arm, Attribute, Error, Fields, Ident, LitStr, Variant, parse_quote};

use super::accessor::Accessor;

mod value;

use value::Value;

pub struct VariantName {
    variant: Ident,
    text: LitStr,
    values: Vec<Value>,
}

impl VariantName {
    const ATTRIBUTE: &'static str = "name";
    const ACCESSOR_ON_VARIANT: &'static str = "accessor belongs on the enum, not on a variant";
    const VALUE: &'static str = "value";
    const UNIT_ONLY: &'static str = "a Named variant carries no fields";
    const REPEATED: &'static str = "a Named variant takes one #[name(\"..\")]";
    const UNNAMED: &'static str = "a Named variant needs #[name(\"..\")]";
    const MISSING: &'static str = "a Named variant needs #[value(accessor = ..)] for every accessor";
    const TWICE: &'static str = "a Named variant gives each accessor one #[value(accessor = ..)]";

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

    pub fn parse_arm(&self, owner: &Ident) -> Arm {
        let variant = &self.variant;
        let text = &self.text;
        parse_quote!(#text => ::core::result::Result::Ok(#owner::#variant))
    }

    pub fn ordinal(&self, owner: &Ident, index: usize) -> TokenStream {
        let variant = &self.variant;
        quote!(#owner::#variant => #index)
    }

    pub fn accessor_arm(&self, owner: &Ident, accessor: &Accessor) -> TokenStream {
        let variant = &self.variant;
        let lit = self.values.iter().find(|value| accessor.key() == value.key()).map(Value::lit);
        quote!(#owner::#variant => #lit)
    }

    pub fn all(variants: Punctuated<Variant, Comma>) -> syn::Result<Vec<VariantName>> {
        variants.into_iter().map(VariantName::try_from).collect()
    }

    pub fn supplies(&self, accessors: &[Accessor]) -> syn::Result<()> {
        self.values.iter().try_for_each(|value| value.declared_in(accessors))?;
        accessors.iter().try_for_each(|accessor| self.once(accessor))
    }

    fn once(&self, accessor: &Accessor) -> syn::Result<()> {
        let supplied = self.values.iter().filter(|value| accessor.key() == value.key()).collect::<Vec<_>>();
        supplied.first().ok_or_else(|| Error::new_spanned(&self.variant, Self::MISSING))?;
        supplied.get(1).map_or(Ok(()), |second| Err(Error::new_spanned(second.accessor(), Self::TWICE)))
    }

    fn named(variant: Variant) -> syn::Result<VariantName> {
        variant
            .attrs
            .iter()
            .find(|attribute| attribute.path().is_ident(Accessor::ATTRIBUTE))
            .map_or(Ok(()), |attribute| Err(Error::new_spanned(attribute, Self::ACCESSOR_ON_VARIANT)))?;
        let names =
            variant.attrs.iter().filter(|attribute| attribute.path().is_ident(Self::ATTRIBUTE)).collect::<Vec<_>>();
        names.get(1).map_or(Ok(()), |second| Err(Error::new_spanned(second, Self::REPEATED)))?;
        let text = names
            .first()
            .ok_or_else(|| Error::new_spanned(&variant.ident, Self::UNNAMED))
            .and_then(|attribute| attribute.parse_args::<LitStr>())?;
        let values = variant
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::VALUE))
            .map(Attribute::parse_args::<Value>)
            .collect::<syn::Result<Vec<Value>>>()?;
        Ok(VariantName { variant: variant.ident, text, values })
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
