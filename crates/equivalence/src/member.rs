use std::fmt;

pub trait Keyed {
    type Key: Ord + fmt::Debug;

    fn key(&self) -> Self::Key;
}

pub trait Nested {
    fn lies_within<'outer>(&self, outer: &'outer Self) -> Option<&'outer Self>;
}
