use syn::punctuated::Punctuated;

use crate::syntax::Syntax;
use syn::{Attribute, ItemImpl, Meta, MetaList, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Walk {
    Entered,
    Skipped,
}

impl Walk {
    const DECIDING: [(&'static str, Self); 2] = [("all", Self::Skipped), ("any", Self::Entered)];

    #[must_use]
    pub fn inherent_impls(item: &ItemImpl) -> Self {
        Syntax::inherent(item).map_or(Self::Skipped, |_| Self::Entered)
    }

    #[must_use]
    pub fn outside_tests(attributes: &[Attribute]) -> Self {
        Self::skipped_by_any(attributes.iter().map(Self::of_attribute))
    }

    pub fn enter(attributes: &[Attribute], visit: impl FnOnce()) {
        match Self::outside_tests(attributes) {
            Self::Entered => visit(),
            Self::Skipped => {}
        }
    }

    fn of_attribute(attribute: &Attribute) -> Self {
        Some(attribute)
            .filter(|attribute| attribute.path().is_ident("cfg"))
            .and_then(|attribute| attribute.parse_args::<Meta>().ok())
            .map_or(Self::Entered, |condition| Self::of_condition(&condition))
    }

    fn of_condition(condition: &Meta) -> Self {
        match condition {
            Meta::Path(path) => {
                path.get_ident().filter(|ident| *ident == "test").map_or(Self::Entered, |_| Self::Skipped)
            }
            Meta::List(list) => list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .map_or(Self::Entered, |nested| Self::of_nested(list, &nested)),
            Meta::NameValue(_) => Self::Entered,
        }
    }

    fn of_nested(list: &MetaList, nested: &Punctuated<Meta, Token![,]>) -> Self {
        let walks: Vec<Self> = nested.iter().map(Self::of_condition).collect();
        Self::DECIDING
            .into_iter()
            .find(|(junction, _)| list.path.is_ident(junction))
            .and_then(|(_, deciding)| walks.iter().find(|walk| **walk == deciding).or(walks.first()))
            .copied()
            .unwrap_or(Self::Entered)
    }

    fn skipped_by_any(walks: impl Iterator<Item = Self>) -> Self {
        walks.into_iter().find(|walk| *walk == Self::Skipped).unwrap_or(Self::Entered)
    }
}
