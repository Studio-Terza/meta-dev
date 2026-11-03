use std::hash::{DefaultHasher, Hash, Hasher};

use newtype::{Arbitrary, Newtype};

use super::ShapeToken;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShapeKey(u64);

impl From<&[ShapeToken]> for ShapeKey {
    fn from(tokens: &[ShapeToken]) -> ShapeKey {
        let mut hasher = DefaultHasher::new();
        tokens.hash(&mut hasher);
        ShapeKey::new(hasher.finish())
    }
}
