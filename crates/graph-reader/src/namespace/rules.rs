use std::collections::BTreeSet;

use lint_layout::Name;
use syntax::{Macro, TokenTree};

#[derive(Debug)]
pub(crate) struct TokenNames;

impl TokenNames {
    pub(crate) fn named(trees: impl Iterator<Item = TokenTree>) -> BTreeSet<Name> {
        trees.flat_map(TokenNames::tree_names).collect()
    }

    fn tree_names(tree: TokenTree) -> BTreeSet<Name> {
        match tree {
            TokenTree::Ident(ident) => BTreeSet::from([Name::from(&ident)]),
            TokenTree::Group(group) => TokenNames::named(group.stream().into_iter()),
            TokenTree::Punct(_) | TokenTree::Literal(_) => BTreeSet::new(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Written(BTreeSet<Name>);

impl From<&Macro> for Written {
    fn from(invoked: &Macro) -> Written {
        Written(TokenNames::named(invoked.tokens.clone().into_iter()))
    }
}

impl Written {
    pub(crate) fn names(&self) -> &BTreeSet<Name> {
        &self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RulesDefinition;
