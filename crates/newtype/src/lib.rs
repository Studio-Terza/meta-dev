mod boundary;
pub mod kind;

pub use boundary::{Checked, Counter, Newtype, Validity, Wrapping};
pub use newtype_derive::{Arbitrary, CheckedDeserialize, Deserialize, Display, Newtype, Serialize};

pub use proptest;
pub use serde;
