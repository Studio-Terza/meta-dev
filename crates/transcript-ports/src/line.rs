use crate::{Record, Skipped, TaskId, Unrecognised};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Said(Record),
    Ended(TaskId),
    Skipped(Skipped),
    Unrecognised(Unrecognised),
}
