use std::collections::BTreeSet;

use proc_macro2::{TokenStream, TokenTree};

use crate::kind::Kind;
use lint_layout::Name;

#[derive(Debug)]
pub(crate) struct Mentions;

impl Mentions {
    const SEPARATOR: char = ',';

    pub(crate) fn beyond_title(tokens: TokenStream, kind: Kind) -> BTreeSet<Name> {
        match kind.titled() {
            Some(_) => Self::named(tokens.into_iter().skip_while(|tree| Self::separator(tree).is_none()).skip(1)),
            None => Self::named(tokens.into_iter()),
        }
    }

    fn named(trees: impl Iterator<Item = TokenTree>) -> BTreeSet<Name> {
        trees.flat_map(Self::tree_names).collect()
    }

    fn tree_names(tree: TokenTree) -> BTreeSet<Name> {
        match tree {
            TokenTree::Ident(ident) => BTreeSet::from([Name::from(&ident)]),
            TokenTree::Group(group) => Self::named(group.stream().into_iter()),
            _ => BTreeSet::new(),
        }
    }

    fn separator(tree: &TokenTree) -> Option<&TokenTree> {
        match tree {
            TokenTree::Punct(punct) => (punct.as_char() == Self::SEPARATOR).then_some(tree),
            _ => None,
        }
    }
}
