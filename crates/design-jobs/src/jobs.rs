mod boundary;
mod state;

pub use boundary::OpenJobs;
pub use state::{JobState, job};

use design_model::JobId;
use store_ports::{Edge, Moved};

impl JobState {
    #[must_use]
    pub fn closing(self, job: JobId) -> Moved<JobState> {
        Moved::new(job, Edge::new(JobState::Open, self))
    }
}
