use std::iter::successors;

use proc_macro2::{Ident, Punct, TokenStream, TokenTree};

use lint_core::Syntax;

pub struct AbsolutePath {
    crate_ident: Ident,
    item: Ident,
}

struct Cursor<'trees> {
    before: Option<&'trees TokenTree>,
    ahead: &'trees [TokenTree],
}

impl AbsolutePath {
    const COLON: char = ':';
    const SPAN: usize = 6;

    pub fn walk(tokens: &TokenStream, visit: impl FnMut(&AbsolutePath)) {
        Self::in_tokens(tokens).iter().for_each(visit);
    }

    pub fn reach(&self, mut visit: impl FnMut(&Ident, &Ident)) {
        visit(&self.crate_ident, &self.item);
    }

    fn in_tokens(tokens: &TokenStream) -> Vec<AbsolutePath> {
        let trees: Vec<TokenTree> = tokens.clone().into_iter().collect();
        let here = Cursor::over(&trees).filter_map(|cursor| cursor.path());
        let nested = trees.iter().filter_map(Syntax::group).flat_map(|group| Self::in_tokens(&group.stream()));
        here.chain(nested).collect()
    }

    fn read([first, second, crate_tree, third, fourth, item_tree]: &[TokenTree; Self::SPAN]) -> Option<AbsolutePath> {
        Self::colon(first)
            .zip(Self::colon(second))
            .zip(Self::colon(third).zip(Self::colon(fourth)))
            .and(Syntax::token_ident(crate_tree).zip(Syntax::token_ident(item_tree)))
            .map(|(crate_ident, item)| AbsolutePath { crate_ident: crate_ident.clone(), item: item.clone() })
    }

    fn colon(tree: &TokenTree) -> Option<&Punct> {
        Syntax::punct(tree).filter(|punct| punct.as_char() == Self::COLON)
    }
}

impl<'trees> Cursor<'trees> {
    fn over(trees: &'trees [TokenTree]) -> impl Iterator<Item = Cursor<'trees>> {
        successors(Some(Cursor { before: None, ahead: trees }), Cursor::advance)
    }

    fn advance(&self) -> Option<Cursor<'trees>> {
        self.ahead.split_first().map(|(head, rest)| Cursor { before: Some(head), ahead: rest })
    }

    fn path(&self) -> Option<AbsolutePath> {
        match self.segment_before() {
            Some(_) => None,
            None => self.ahead.first_chunk().and_then(AbsolutePath::read),
        }
    }

    fn segment_before(&self) -> Option<&Ident> {
        self.before
            .and_then(Syntax::token_ident)
            .filter(|ident| syn::parse2::<Ident>(std::iter::once(TokenTree::from((*ident).clone())).collect()).is_ok())
    }
}
