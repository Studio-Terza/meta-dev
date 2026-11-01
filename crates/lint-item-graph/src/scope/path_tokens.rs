use proc_macro2::{TokenStream, TokenTree};

#[derive(Debug)]
pub struct PathTokens(Vec<TokenTree>);

impl PathTokens {
    const SEPARATOR: char = ':';
    const SEPARATOR_LENGTH: usize = 2;
    const SEGMENT_LENGTH: usize = 3;

    pub fn new(tokens: TokenStream) -> PathTokens {
        PathTokens(tokens.into_iter().collect())
    }

    pub fn tokens(&self) -> &[TokenTree] {
        &self.0
    }

    pub fn root_index(&self, index: usize) -> Option<usize> {
        if self.separator_before_index(index) {
            index
                .checked_sub(Self::SEGMENT_LENGTH)
                .filter(|before| matches!(self.0.get(*before), Some(TokenTree::Ident(_))))
                .and_then(|before| self.root_index(before))
        } else {
            Some(index)
        }
    }

    fn separator_before_index(&self, index: usize) -> bool {
        (1..=Self::SEPARATOR_LENGTH).all(|back| {
            index
                .checked_sub(back)
                .and_then(|at| self.0.get(at))
                .is_some_and(|token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == Self::SEPARATOR))
        })
    }
}
