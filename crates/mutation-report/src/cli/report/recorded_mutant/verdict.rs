use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum Verdict {
    Success,
    CaughtMutant,
    MissedMutant,
    Unviable,
    Timeout,
    Failure,
}
