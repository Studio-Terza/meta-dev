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

    pub fn roots(&self) -> Vec<Option<&TokenTree>> {
        self.0.iter().fold(Vec::new(), |mut found, _| {
            let root = self.next_root(&found);
            found.push(root);
            found
        })
    }

    fn next_root<'tokens>(&'tokens self, found: &[Option<&'tokens TokenTree>]) -> Option<&'tokens TokenTree> {
        let index = found.len();
        (1..=Self::SEPARATOR_LENGTH)
            .map(|back| index.checked_sub(back).and_then(|at| self.0.get(at)).and_then(Self::separator))
            .collect::<Option<Vec<&TokenTree>>>()
            .map_or(self.0.get(index), |_| {
                index
                    .checked_sub(Self::SEGMENT_LENGTH)
                    .filter(|before| matches!(self.0.get(*before), Some(TokenTree::Ident(_))))
                    .and_then(|before| found.get(before).copied().flatten())
            })
    }

    fn separator(token: &TokenTree) -> Option<&TokenTree> {
        Some(token).filter(|token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == Self::SEPARATOR))
    }
}
