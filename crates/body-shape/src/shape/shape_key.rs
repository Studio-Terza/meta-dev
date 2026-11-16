use std::hash::{DefaultHasher, Hash, Hasher};

use newtype::{Arbitrary, Newtype};

use super::Labels;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ShapeKey(u64);

impl From<&Labels> for ShapeKey {
    fn from(labels: &Labels) -> ShapeKey {
        let mut hasher = DefaultHasher::new();
        labels.hash(&mut hasher);
        ShapeKey::new(hasher.finish())
    }
}
