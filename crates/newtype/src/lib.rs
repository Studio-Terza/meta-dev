mod boundary;
mod plain;
mod validated;

pub use boundary::Counter;
pub use newtype_derive::{Arbitrary, Display, Newtype};
pub use plain::Newtype;
pub use validated::Validated;

pub use proptest;
