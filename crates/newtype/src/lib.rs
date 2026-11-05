#![doc = include_str!("named.md")]

mod boundary;

pub use boundary::{Counter, Newtype, Unnamed};
pub use newtype_derive::{Arbitrary, Display, Named, Newtype};

pub use proptest;
