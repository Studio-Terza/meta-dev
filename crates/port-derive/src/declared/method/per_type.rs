use proc_macro2::TokenStream;
use syn::{Attribute, Meta, MetaList, TraitItemFn};

pub struct PerType;

impl PerType {
    const ATTRIBUTE: &'static str = "op";

    pub fn stripped(function: &mut TraitItemFn) -> Option<TokenStream> {
        function
            .attrs
            .iter()
            .position(|attribute| Self::listed(attribute).is_some())
            .map(|position| function.attrs.remove(position))
            .and_then(|attribute| Self::listed(&attribute).map(|list| list.tokens.clone()))
    }

    fn listed(attribute: &Attribute) -> Option<&MetaList> {
        match &attribute.meta {
            Meta::List(list) => Some(list).filter(|candidate| candidate.path.is_ident(Self::ATTRIBUTE)),
            Meta::Path(_) | Meta::NameValue(_) => None,
        }
    }
}
