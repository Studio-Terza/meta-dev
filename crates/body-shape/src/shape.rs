mod labels;
mod literals;
mod mark;
mod neighbours;
mod position;
mod raw_token;
mod shape_key;
mod shape_token;
mod slot;
mod slots;
mod spelling;

use equivalence::Equivalence;
use syntax::TokenStream;

use crate::token_count::TokenCount;
use neighbours::Neighbours;
use raw_token::RawToken;
pub(crate) use shape_key::ShapeKey;
use slots::Slots;

pub(crate) use labels::Code;
pub use labels::Labels;
pub use literals::Literals;
pub(crate) use shape_token::ShapeToken;
pub(crate) use spelling::{Sign, Spelled, Spelling, Spoken};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Shape {
    size: TokenCount,
    key: ShapeKey,
}

impl Shape {
    #[must_use]
    pub fn size(self) -> TokenCount {
        self.size
    }

    pub(crate) fn ranged(size: TokenCount, key: ShapeKey) -> Shape {
        Shape { size, key }
    }

    fn read(raw: &[RawToken], equivalence: Equivalence) -> Shape {
        let mut slots = Slots::default();
        let shaped: Vec<ShapeToken<'_>> = raw
            .iter()
            .zip(Neighbours::all(raw))
            .filter_map(|(token, neighbours)| token.shaped(neighbours.position(), &mut slots, equivalence))
            .collect();
        Shape { key: ShapeKey::from(shaped.as_slice()), size: TokenCount::new(shaped.len()) }
    }
}

#[derive(Clone, Debug)]
pub struct Reading {
    shape: Shape,
    literals: Literals,
    labels: Labels,
}

impl Reading {
    #[must_use]
    pub fn erasing(tokens: TokenStream, equivalence: Equivalence) -> Reading {
        let raw = RawToken::flattened(tokens);
        let shape = Shape::read(&raw, equivalence);
        let signs: Vec<Sign> = Neighbours::all(&raw).map(Neighbours::sign).collect();
        let literals = raw.into_iter().zip(signs).filter_map(|(token, sign)| token.spelled(sign)).collect();
        Reading { shape, literals, labels: Labels::default() }
    }

    pub(crate) fn new(shape: Shape, literals: Literals, labels: Labels) -> Reading {
        Reading { shape, literals, labels }
    }

    #[must_use]
    pub fn labels(&self) -> &Labels {
        &self.labels
    }

    #[must_use]
    pub fn shape(&self) -> Shape {
        self.shape
    }

    #[must_use]
    pub fn into_literals(self) -> Literals {
        self.literals
    }
}
