use syn::{Ident, LitStr, Variant};

pub struct Row {
    variant: Ident,
    name: LitStr,
}

impl Row {
    const ATTRIBUTE: &'static str = "name";

    pub fn declared(variant: &Variant) -> syn::Result<Vec<Row>> {
        variant
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::ATTRIBUTE))
            .map(|attribute| attribute.parse_args().map(|name| Row { variant: variant.ident.clone(), name }))
            .collect()
    }

    pub fn variant(&self) -> &Ident {
        &self.variant
    }

    pub fn name(&self) -> &LitStr {
        &self.name
    }
}
