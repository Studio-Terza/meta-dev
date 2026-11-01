use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Verdict {
    Success,
    CaughtMutant,
    MissedMutant,
    Unviable,
    Timeout,
    Failure,
}
