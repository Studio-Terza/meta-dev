use crate::ids::Timestamp;
use crate::line_hash::LineHash;
use serde::{Deserialize, Serialize};

/// Where an escape came from when it did not start in the log: a line of the side file of
/// escapes from builds not verified to be from main, and when that line was written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Imported {
    pub line: LineHash,
    pub written: Timestamp,
}
