mod boundary;

use design_model::JobId;
use named_derive::Named;
use store_ports::{Edge, Lifecycle, Moved};

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobState {
    Open,
    Applied,
    Repeated,
    Flailed,
    Stalled,
    Interrupted,
}

impl Lifecycle for JobState {
    const EDGES: &'static [Edge<JobState>] = &[
        Edge::new(JobState::Open, JobState::Applied),
        Edge::new(JobState::Open, JobState::Repeated),
        Edge::new(JobState::Open, JobState::Flailed),
        Edge::new(JobState::Open, JobState::Stalled),
        Edge::new(JobState::Open, JobState::Interrupted),
    ];
}

impl JobState {
    #[must_use]
    pub fn closing(self, job: JobId) -> Moved<JobState> {
        Moved::new(job, Edge::new(JobState::Open, self))
    }
}
