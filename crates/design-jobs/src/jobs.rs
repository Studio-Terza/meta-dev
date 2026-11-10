mod boundary;

use design_model::JobId;
use named_derive::Named;
use port_core::Op;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobState {
    #[name("open")]
    Open,
    #[name("applied")]
    Applied,
    #[name("repeated")]
    Repeated,
    #[name("flailed")]
    Flailed,
    #[name("stalled")]
    Stalled,
    #[name("interrupted")]
    Interrupted,
}

#[derive(Clone, Copy, Debug, Op)]
pub struct JobClosed {
    job: JobId,
    state: JobState,
}

impl JobClosed {
    #[must_use]
    pub fn new(job: JobId, state: JobState) -> JobClosed {
        JobClosed { job, state }
    }
}
