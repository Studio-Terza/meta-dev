mod boundary;
mod extent;

pub use boundary::{Counter, Exit, Listed, Newtype, Unnamed};
pub use extent::{Extent, Filled, Measured};
#[doc = include_str!("named.md")]
pub use newtype_derive::Named;
pub use newtype_derive::{Arbitrary, Display, Newtype};

pub use newtype_derive::VariantArray;
pub use proptest;
pub use strum_trait::*;

mod strum_trait {
    pub use strum::VariantArray;
}
