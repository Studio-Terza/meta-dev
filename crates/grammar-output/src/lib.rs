#![deny(clippy::wildcard_enum_match_arm)]

mod boundary;
pub mod field;

pub(crate) use boundary::{Decimal, Field, Kind, Numbered, Token, Unwrapping};
pub use boundary::{Fields, Unread};
