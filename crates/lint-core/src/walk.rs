use proc_macro2::{TokenStream, TokenTree};
use syn::{Attribute, ItemImpl};

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
        match attribute.meta.require_list() {
            Ok(list) if attribute.path().is_ident("cfg") => Self::of_tokens(list.tokens.clone()),
            _ => Walk::Entered,
        }
    }

    fn of_tokens(tokens: TokenStream) -> Walk {
        Self::skipped_by_any(tokens.into_iter().map(|token| match token {
            TokenTree::Ident(ident) if ident == "test" => Walk::Skipped,
            TokenTree::Group(group) => Self::of_tokens(group.stream()),
            TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => Walk::Entered,
        }))
    }

    fn skipped_by_any(walks: impl Iterator<Item = Walk>) -> Walk {
        walks.into_iter().find(|walk| *walk == Walk::Skipped).unwrap_or(Walk::Entered)
    }
}
