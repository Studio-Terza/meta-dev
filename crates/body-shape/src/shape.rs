mod mark;
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

use shape_key::ShapeKey;
pub(crate) use shape_token::ShapeToken;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Shape {
    key: ShapeKey,
    size: TokenCount,
}

impl Shape {
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
        Shape { key: ShapeKey::from(shaped.as_slice()), size: TokenCount::new(shaped.len()) }
    }
}
