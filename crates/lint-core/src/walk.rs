use syn::punctuated::Punctuated;
use syn::{Attribute, ItemImpl, Meta, MetaList, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Walk {
    Entered,
    Skipped,
}

impl Walk {
    #[must_use]
    pub fn inherent_impls(item: &ItemImpl) -> Walk {
        match item.trait_ {
            None => Walk::Entered,
            Some(_) => Walk::Skipped,
        }
    }

    #[must_use]
    pub fn outside_tests(attributes: &[Attribute]) -> Walk {
        Self::skipped_by_any(attributes.iter().map(Self::of_attribute))
    }

    fn of_attribute(attribute: &Attribute) -> Walk {
        if attribute.path().is_ident("cfg") {
            attribute.parse_args::<Meta>().map_or(Walk::Entered, |condition| Self::of_condition(&condition))
        } else {
            Walk::Entered
        }
    }

    fn of_condition(condition: &Meta) -> Walk {
        match condition {
            Meta::Path(path) if path.is_ident("test") => Walk::Skipped,
            Meta::List(list) => list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .map_or(Walk::Entered, |nested| Self::of_nested(list, &nested)),
            Meta::Path(_) | Meta::NameValue(_) => Walk::Entered,
        }
    }

    fn of_nested(list: &MetaList, nested: &Punctuated<Meta, Token![,]>) -> Walk {
        let walks: Vec<Walk> = nested.iter().map(Self::of_condition).collect();
        if list.path.is_ident("all") {
            Self::skipped_by_any(walks.into_iter())
        } else if list.path.is_ident("any") && !walks.is_empty() && walks.iter().all(|walk| *walk == Walk::Skipped) {
            Walk::Skipped
        } else {
            Walk::Entered
        }
    }

    fn skipped_by_any(walks: impl Iterator<Item = Walk>) -> Walk {
        walks.into_iter().find(|walk| *walk == Walk::Skipped).unwrap_or(Walk::Entered)
    }
}
