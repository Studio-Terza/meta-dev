mod boundary;
mod stored;

pub use boundary::{Counter, Newtype};
pub use newtype_derive::{Arbitrary, Display, Newtype, Spelled};
pub use stored::{Inner, Optional, Spelled, Stored};

pub use proptest;
