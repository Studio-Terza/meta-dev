use std::collections::BTreeSet;

use proc_macro2::{TokenStream, TokenTree};

use crate::coverage::kind::Kind;
use lint_layout::Name;

#[derive(Debug)]
pub(crate) struct Mentions;

impl Mentions {
    const SEPARATOR: char = ',';

    pub(crate) fn beyond_title(tokens: TokenStream, kind: Kind) -> BTreeSet<Name> {
        match kind {
            Kind::Roundtrip | Kind::Law => Self::named(
                tokens
                    .into_iter()
                    .skip_while(|tree| !matches!(tree, TokenTree::Punct(punct) if punct.as_char() == Self::SEPARATOR))
                    .skip(1),
            ),
            Kind::Proptest | Kind::Other => Self::named(tokens.into_iter()),
        }
    }

    fn named(trees: impl Iterator<Item = TokenTree>) -> BTreeSet<Name> {
        trees
            .flat_map(|tree| match tree {
                TokenTree::Ident(ident) => BTreeSet::from([Name::from(&ident)]),
                TokenTree::Group(group) => Self::named(group.stream().into_iter()),
                _ => BTreeSet::new(),
            })
            .collect()
    }
}
