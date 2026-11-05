mod boundary;
mod extent;

pub use boundary::{Counter, Exit, Newtype, Unnamed};
pub use extent::{Extent, Filled, Measured};
#[doc = include_str!("named.md")]
pub use newtype_derive::Named;
pub use newtype_derive::{Arbitrary, Display, Newtype};

pub use proptest;
