mod bits;
mod plain;
mod validated;

pub use bits::Counter;
pub use newtype_derive::{Arbitrary, Newtype};
pub use plain::Newtype;
pub use validated::Validated;
