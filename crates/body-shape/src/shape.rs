mod neighbours;
mod position;
mod raw_token;
mod shape_key;
mod shape_token;
mod slot;
mod slots;

use proc_macro2::TokenStream;

use crate::token_count::TokenCount;
use neighbours::Neighbours;
use raw_token::RawToken;
use slots::Slots;

pub use shape_key::ShapeKey;
pub(crate) use shape_token::ShapeToken;

#[derive(Clone, Copy, Debug)]
pub struct Shape {
    key: ShapeKey,
    size: TokenCount,
}

impl Shape {
    #[must_use]
    pub fn key(self) -> ShapeKey {
        self.key
    }

    #[must_use]
    pub fn size(self) -> TokenCount {
        self.size
    }
}

impl From<TokenStream> for Shape {
    fn from(tokens: TokenStream) -> Shape {
        let raw = RawToken::flattened(tokens);
        let mut slots = Slots::default();
        let shaped: Vec<ShapeToken> = raw
            .iter()
            .enumerate()
            .filter_map(|(index, token)| token.shaped(Neighbours::around(&raw, index).position(), &mut slots))
            .collect();
        Shape { key: ShapeKey::from(shaped.as_slice()), size: TokenCount::new(shaped.len()) }
    }
}
