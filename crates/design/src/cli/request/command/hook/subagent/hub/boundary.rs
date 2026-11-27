use dispatch_ports::{Call, Clock, Deadline, Dispatch, Ran, Shell, Timeout};
use invariants::Ignoring;

use super::super::boundary::JobId;
use super::{Hub, Link};

impl From<String> for Link {
    fn from(link: String) -> Link {
        Link(JobId::linked(&link))
    }
}
use crate::cli::request::ignored::Ignored;

impl<D: Dispatch, C: Clock, G: Shell> Hub<'_, D, C, G> {
    pub(super) fn ran(&self, arguments: &[&str], timeout: Timeout) -> Option<Ran> {
        let spoken = arguments.iter().map(|word| (*word).to_owned()).collect();
        let deadline = Deadline::after(self.crew.now(), timeout);
        let call = Call::new(Hub::<D, C, G>::PROGRAM, spoken, self.directory.to_path_buf(), deadline);
        self.crew.ran(&call).ignoring(Ignored::Unrun)
    }

    pub(crate) fn log(&self, job: JobId) -> String {
        let ran = self.ran(&["run", "view", "--job", &job.to_string(), "--log-failed"], Self::QUICK);
        ran.as_ref().and_then(Ran::succeeded).map_or_else(|| Hub::<D, C, G>::UNLOGGED.to_owned(), ToString::to_string)
    }
}
