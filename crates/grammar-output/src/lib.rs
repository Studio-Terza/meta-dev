#![deny(clippy::wildcard_enum_match_arm)]

mod boundary;
pub mod field;
mod record;

pub(crate) use boundary::{Decimal, Field, Kind, Numbered, Token, Unwrapping};
pub use boundary::{Fields, Source};
pub use record::{Parsed, Record};
pub use record_derive::Record;
