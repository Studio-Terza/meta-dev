use proc_macro2::{Group, Ident, Punct, TokenStream, TokenTree};
use quote::quote;

#[derive(Debug)]
pub struct Rewritten(TokenStream);

#[derive(Clone, Copy, Debug)]
struct Tree<'tree>(&'tree TokenTree);

impl Rewritten {
    fn rewritten(tokens: TokenStream) -> TokenStream {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        let colons = trees.iter().skip(1).map(|tree| Tree(tree).colon()).chain([None]);
        trees.iter().zip(colons).map(|(tree, after)| Tree(tree).replaced(after)).collect()
    }

    fn regrouped(group: &Group) -> Group {
        let mut regrouped = Group::new(group.delimiter(), Self::rewritten(group.stream()));
        regrouped.set_span(group.span());
        regrouped
    }
}

impl<'tree> Tree<'tree> {
    const SQLX: &'static str = "sqlx";
    const COLON: char = ':';

    fn colon(self) -> Option<&'tree Punct> {
        match self.0 {
            TokenTree::Punct(punct) => Some(punct).filter(|punct| punct.as_char() == Self::COLON),
            TokenTree::Group(_) | TokenTree::Ident(_) | TokenTree::Literal(_) => None,
        }
    }

    fn sqlx(self) -> Option<&'tree Ident> {
        match self.0 {
            TokenTree::Ident(ident) => Some(ident).filter(|ident| *ident == Self::SQLX),
            TokenTree::Group(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => None,
        }
    }

    fn replaced(self, after: Option<&Punct>) -> TokenStream {
        after.and(self.sqlx()).map_or_else(|| self.kept(), |_| quote!(store_ports::__sqlx))
    }

    fn kept(self) -> TokenStream {
        match self.0 {
            TokenTree::Group(group) => TokenTree::Group(Rewritten::regrouped(group)).into(),
            TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => self.0.clone().into(),
        }
    }
}

impl From<TokenStream> for Rewritten {
    fn from(tokens: TokenStream) -> Rewritten {
        Rewritten(Rewritten::rewritten(tokens))
    }
}

impl From<Rewritten> for TokenStream {
    fn from(rewritten: Rewritten) -> TokenStream {
        rewritten.0
    }
}
