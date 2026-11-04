mod boundary;
mod plain;

pub use boundary::Counter;
pub use newtype_derive::{Arbitrary, Display, Newtype};
pub use plain::Newtype;

pub use proptest;
