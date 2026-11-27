use crate::{Record, Skipped, Unrecognised};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Said(Record),
    Skipped(Skipped),
    Unrecognised(Unrecognised),
}
