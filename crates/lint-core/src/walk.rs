use syn::punctuated::Punctuated;

use crate::syntax::Syntax;
use syn::{Attribute, ItemImpl, Meta, MetaList, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Walk {
    Entered,
    Skipped,
}

impl Walk {
    const DECIDING: [(&'static str, Walk); 2] = [("all", Walk::Skipped), ("any", Walk::Entered)];

    #[must_use]
    pub fn inherent_impls(item: &ItemImpl) -> Walk {
        Syntax::inherent(item).map_or(Walk::Skipped, |_| Walk::Entered)
    }

    #[must_use]
    pub fn outside_tests(attributes: &[Attribute]) -> Walk {
        Self::skipped_by_any(attributes.iter().map(Self::of_attribute))
    }

    pub fn enter(attributes: &[Attribute], visit: impl FnOnce()) {
        match Self::outside_tests(attributes) {
            Walk::Entered => visit(),
            Walk::Skipped => {}
        }
    }

    fn of_attribute(attribute: &Attribute) -> Walk {
        Some(attribute)
            .filter(|attribute| attribute.path().is_ident("cfg"))
            .and_then(|attribute| attribute.parse_args::<Meta>().ok())
            .map_or(Walk::Entered, |condition| Self::of_condition(&condition))
    }

    fn of_condition(condition: &Meta) -> Walk {
        match condition {
            Meta::Path(path) => {
                path.get_ident().filter(|ident| *ident == "test").map_or(Walk::Entered, |_| Walk::Skipped)
            }
            Meta::List(list) => list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .map_or(Walk::Entered, |nested| Self::of_nested(list, &nested)),
            Meta::NameValue(_) => Walk::Entered,
        }
    }

    fn of_nested(list: &MetaList, nested: &Punctuated<Meta, Token![,]>) -> Walk {
        let walks: Vec<Walk> = nested.iter().map(Self::of_condition).collect();
        Self::DECIDING
            .into_iter()
            .find(|(junction, _)| list.path.is_ident(junction))
            .and_then(|(_, deciding)| walks.iter().find(|walk| **walk == deciding).or(walks.first()))
            .copied()
            .unwrap_or(Walk::Entered)
    }

    fn skipped_by_any(walks: impl Iterator<Item = Walk>) -> Walk {
        walks.into_iter().find(|walk| *walk == Walk::Skipped).unwrap_or(Walk::Entered)
    }
}
