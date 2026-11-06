use std::error::Error;

use proc_macro2::{Group, Ident, Punct, TokenStream, TokenTree};
use quote::quote;

#[derive(Debug)]
pub struct Rewritten(TokenStream);

impl Rewritten {
    const SQLX: &'static str = "sqlx";
    const COLON: char = ':';

    pub fn refused(failure: Box<dyn Error>) -> TokenStream {
        failure.downcast::<syn::Error>().map_or_else(
            |other| {
                let message = other.to_string();
                quote!(::std::compile_error!(#message))
            },
            |parsed| parsed.to_compile_error(),
        )
    }

    fn rewritten(tokens: TokenStream) -> TokenStream {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        let colons = trees.iter().skip(1).map(Self::colon).chain([None]);
        trees.iter().zip(colons).map(|(tree, after)| Self::replaced(tree, after)).collect()
    }

    fn colon(tree: &TokenTree) -> Option<&Punct> {
        match tree {
            TokenTree::Punct(punct) => Some(punct).filter(|punct| punct.as_char() == Self::COLON),
            TokenTree::Group(_) | TokenTree::Ident(_) | TokenTree::Literal(_) => None,
        }
    }

    fn sqlx(tree: &TokenTree) -> Option<&Ident> {
        match tree {
            TokenTree::Ident(ident) => Some(ident).filter(|ident| *ident == Self::SQLX),
            TokenTree::Group(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => None,
        }
    }

    fn replaced(tree: &TokenTree, after: Option<&Punct>) -> TokenStream {
        after
            .and(Self::sqlx(tree))
            .map_or_else(|| Self::kept(tree), |_| quote!(store_ports::__sqlx))
    }

    fn kept(tree: &TokenTree) -> TokenStream {
        match tree {
            TokenTree::Group(group) => TokenTree::Group(Self::regrouped(group)).into(),
            TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => tree.clone().into(),
        }
    }

    fn regrouped(group: &Group) -> Group {
        let mut regrouped = Group::new(group.delimiter(), Self::rewritten(group.stream()));
        regrouped.set_span(group.span());
        regrouped
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
