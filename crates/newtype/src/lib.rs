#![doc = include_str!("named.md")]

mod boundary;
mod extent;

pub use boundary::{Counter, Exit, Newtype, Unnamed};
pub use extent::{Extent, Filled, Measured};
pub use newtype_derive::{Arbitrary, Display, Named, Newtype};

pub use proptest;
