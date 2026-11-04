mod mark;
mod neighbours;
mod position;
mod raw_token;
mod shape_token;
mod slot;
mod slots;

use std::hash::{DefaultHasher, Hash, Hasher};

use proc_macro2::TokenStream;

use crate::shape_key::ShapeKey;
use crate::token_count::TokenCount;
use neighbours::Neighbours;
use raw_token::RawToken;
use slots::Slots;

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
        let shaped: Vec<ShapeToken<'_>> = raw
            .iter()
            .zip(Neighbours::all(&raw))
            .filter_map(|(token, neighbours)| token.shaped(neighbours.position(), &mut slots))
            .collect();
        let mut hasher = DefaultHasher::new();
        shaped.hash(&mut hasher);
        Shape { key: ShapeKey::new(hasher.finish()), size: TokenCount::new(shaped.len()) }
    }
}
