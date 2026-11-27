#![deny(clippy::wildcard_enum_match_arm)]

mod record;

pub use record::{Parsed, Record, Source};
pub use record_derive::Record;
