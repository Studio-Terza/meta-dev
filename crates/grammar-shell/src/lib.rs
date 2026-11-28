#![deny(clippy::wildcard_enum_match_arm)]

mod boundary;

pub use boundary::{Construct, Directory, Edit, Edited, Editor, Flagged, Nested, Unreadable};
